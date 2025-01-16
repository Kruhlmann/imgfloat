use futures::{SinkExt, StreamExt};
use std::{error::Error, fmt::Display, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio::{net::TcpStream, time::sleep};
use tokio_native_tls::{native_tls, TlsConnector};
use tokio_util::codec::{Framed, LinesCodec};

use crate::twitch::{TwitchApiResponse, TwitchUser, TwitchUserTokens};

use super::{ApplicationController, IrcMessage, RemoteImageBlob};

#[derive(Clone)]
pub struct ChatDaemon {
    pub user: TwitchUser,
    pub tokens: Arc<Mutex<TwitchUserTokens>>,
    messages: Arc<Mutex<Vec<TwitchMessage>>>,
    background_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

#[derive(Debug, serde::Serialize)]
pub struct TwitchMessage {
    username: String,
    message: String,
    blob: Option<RemoteImageBlob>,
}

#[derive(Debug)]
pub enum ChatDaemonError {
    TokenRequestFailed(reqwest::Error),
    InvalidTokenResponse(reqwest::Error),
    UserInfoRequestFailed(reqwest::Error),
    InvalidUserInfoResponse(reqwest::Error),
    NoSuchUser,
}

impl Error for ChatDaemonError {}

impl Display for ChatDaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TokenRequestFailed(e) => write!(f, "{:?}: {e}", self),
            Self::InvalidTokenResponse(e) => write!(f, "{:?}: {e}", self),
            Self::UserInfoRequestFailed(e) => write!(f, "{:?}: {e}", self),
            Self::InvalidUserInfoResponse(e) => write!(f, "{:?}: {e}", self),
            Self::NoSuchUser => write!(f, "{:?}", self),
        }
    }
}

impl ChatDaemon {
    pub async fn new(
        application_controller: &ApplicationController,
        user_token: &str,
    ) -> Result<Self, ChatDaemonError> {
        let client = reqwest::Client::new();
        let tokens = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", &application_controller.client_id),
                ("client_secret", &application_controller.client_secret),
                ("code", &user_token.to_owned()),
                ("grant_type", &"authorization_code".to_owned()),
                ("redirect_uri", &application_controller.redirect_uri),
            ])
            .send()
            .await
            .map_err(ChatDaemonError::TokenRequestFailed)?
            .error_for_status()
            .map_err(ChatDaemonError::TokenRequestFailed)?
            .json::<TwitchUserTokens>()
            .await
            .map_err(ChatDaemonError::InvalidTokenResponse)?;
        let user = client
            .get("https://api.twitch.tv/helix/users")
            .header("Authorization", format!("Bearer {}", tokens.access_token))
            .header("Client-Id", &application_controller.client_id)
            .send()
            .await
            .map_err(ChatDaemonError::UserInfoRequestFailed)?
            .error_for_status()
            .map_err(ChatDaemonError::UserInfoRequestFailed)?
            .json::<TwitchApiResponse<Vec<TwitchUser>>>()
            .await
            .map_err(ChatDaemonError::InvalidUserInfoResponse)?
            .data
            .into_iter()
            .next()
            .ok_or(ChatDaemonError::NoSuchUser)?;

        let mut daemon = Self {
            user,
            tokens: Arc::new(Mutex::new(tokens)),
            messages: Arc::new(Mutex::new(Vec::new())),
            background_task: Arc::new(Mutex::new(None)),
        };

        daemon.start_irc_loop().await?;
        daemon.start_refresh_loop(application_controller).await?;

        Ok(daemon)
    }

    pub async fn start_irc_loop(&self) -> Result<(), ChatDaemonError> {
        let user_login = self.user.login.clone();
        let oauth_token = format!("oauth:{}", self.tokens.lock().await.access_token);
        let messages_ref = self.messages.clone();
        let mut bg_lock = self.background_task.lock().await;

        let handle = tokio::spawn(async move {
            Self::irc_loop(&user_login, &oauth_token, messages_ref)
                .await
                .inspect_err(|error| tracing::error!(?error, "irc loop error"))
                .ok();
        });

        *bg_lock = Some(handle);
        Ok(())
    }

    pub async fn start_refresh_loop(
        &mut self,
        application_controller: &ApplicationController,
    ) -> Result<(), ChatDaemonError> {
        let tokens_ref = self.tokens.clone();
        let client_id = application_controller.client_id.clone();
        let client_secret = application_controller.client_secret.clone();
        let refresh_interval = Duration::from_mins(2);

        let mut bg_lock = self.background_task.lock().await;
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();
            // This has been moved out into this closure to make it easier to handle errors
            let refresh = async || -> Result<(), reqwest::Error> {
                let current_refresh_token = {
                    let tokens = tokens_ref.lock().await;
                    tokens.refresh_token.clone()
                };
                let new_tokens = client
                    .post("https://id.twitch.tv/oauth2/token")
                    .form(&[
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("grant_type", "refresh_token"),
                        ("refresh_token", current_refresh_token.unwrap().as_str()),
                    ])
                    .send()
                    .await
                    .and_then(|r| r.error_for_status())?
                    .json::<TwitchUserTokens>()
                    .await?;
                let mut tokens_lock = tokens_ref.lock().await;
                *tokens_lock = new_tokens;
                Ok(())
            };

            loop {
                sleep(refresh_interval).await;
                refresh()
                    .await
                    .inspect_err(|error| tracing::error!(?error, "unable to update tokens"))
                    .ok();
            }
        });

        *bg_lock = Some(handle);
        Ok(())
    }

    async fn irc_loop(
        user_login: &str,
        oauth_token: &str,
        messages: Arc<Mutex<Vec<TwitchMessage>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let socket = TcpStream::connect(("irc.chat.twitch.tv", 6697)).await?;
        let connector = TlsConnector::from(native_tls::TlsConnector::new()?);
        let tls_stream = connector.connect("irc.chat.twitch.tv", socket).await?;
        let mut framed = Framed::new(tls_stream, LinesCodec::new());
        framed.send(format!("PASS {}", oauth_token)).await?;
        framed.send(format!("NICK {}", user_login)).await?;
        framed
            .send(format!("JOIN {}", format!("#{}", user_login)))
            .await?;

        while let Some(Ok(line)) = framed.next().await {
            match IrcMessage::from(line) {
                IrcMessage::Ping(server) => framed.send(format!("PONG {server}")).await?,
                IrcMessage::PrivMsg { username, message } => {
                    tracing::debug!(?username, ?message, "user irc message");
                    let blob = if message.starts_with("!imgfloat") {
                        let url = message["!imgfloat ".len()..]
                            .trim_end_matches('\u{e0000}')
                            .trim()
                            .to_string();
                        RemoteImageBlob::download(&url).await
                    } else {
                        None
                    };
                    messages.lock().await.push(TwitchMessage {
                        username,
                        message,
                        blob,
                    });
                }
                IrcMessage::Join(username) => tracing::debug!("user {username} joined channel"),
                IrcMessage::NamesList(names) => tracing::debug!("namelist: {names}"),
                IrcMessage::EndOfNames => tracing::debug!("end of namelist"),
                IrcMessage::Numeric { code, user, info } => {
                    tracing::warn!(?code, ?user, ?info, "unknown numeric message")
                }
                IrcMessage::Unknown(m) => tracing::warn!("unknown message: {m}"),
            }
        }

        Ok(())
    }

    pub async fn consume_messages(&self) -> Vec<TwitchMessage> {
        let mut lock = self.messages.lock().await;
        lock.drain(..).collect()
    }
}
