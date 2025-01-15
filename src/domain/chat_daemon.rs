use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use std::{error::Error, fmt::Display, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio::{net::TcpStream, time::sleep};
use tokio_native_tls::{native_tls, TlsConnector};
use tokio_util::codec::{Framed, LinesCodec};

use crate::twitch::{TwitchApiResponse, TwitchUser, TwitchUserTokens};

use super::ApplicationController;

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

#[derive(Debug, serde::Serialize)]
pub struct RemoteImageBlob {
    bytes_base64: String,
    mime_type: String,
}

impl RemoteImageBlob {
    pub async fn download(url: &str) -> Option<Self> {
        tracing::info!(?url, "downloading image");
        let response = reqwest::get(url.trim())
            .await
            .inspect_err(|error| tracing::error!(?url, ?error, "http error when reading image"))
            .ok()?;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|val| val.to_str().ok())?;
        let mime_type = content_type
            .parse::<mime::Mime>()
            .inspect_err(|error| tracing::error!(?error, ?content_type, "invalid content type"))
            .ok()?;
        if mime_type.type_() != mime::IMAGE {
            tracing::error!(?mime_type, "resource isn't an image");
            if mime_type == mime::TEXT_PLAIN || mime_type == mime::TEXT_PLAIN_UTF_8 {
                let response_text = response
                    .text()
                    .await
                    .inspect_err(|error| {
                        tracing::error!(?url, ?error, "http error when reading image response")
                    })
                    .ok()?;
                tracing::debug!(?response_text, "text response received");
            }
            return None;
        }
        let bytes = response
            .bytes()
            .await
            .inspect_err(|error| {
                tracing::error!(?url, ?error, "http error when reading image response")
            })
            .ok()?;
        let bytes_base64 = BASE64_STANDARD.encode(&bytes);
        Some(Self {
            bytes_base64,
            mime_type: mime_type.to_string(),
        })
    }
}

enum IrcMessage {
    Ping(String),
    Numeric {
        code: String,
        user: String,
        info: String,
    },
    PrivMsg {
        username: String,
        message: String,
    },
    Join(String),
    NamesList(String),
    EndOfNames,
    Unknown(String),
}

impl From<String> for IrcMessage {
    fn from(line: String) -> Self {
        if line.starts_with("PING ") {
            if let Some(server) = line.strip_prefix("PING ") {
                let server = server.trim_start_matches(':').to_string();
                return IrcMessage::Ping(server);
            }
        }

        static NUMERIC_RE: &str =
            r"^:(?P<server>[^ ]+)\s(?P<code>\d{3})\s(?P<user>\S+)\s(?P<info>.*)$";
        let numeric = Regex::new(NUMERIC_RE).unwrap();

        if let Some(caps) = numeric.captures(&line) {
            let code = caps["code"].to_string();
            let user = caps["user"].to_string();
            let info = caps["info"].to_string();

            match code.as_str() {
                "353" => {
                    let re_353 = Regex::new(r"^=(?P<channel>#[^ ]+)\s:(?P<names>.*)$").unwrap();
                    if let Some(names_caps) = re_353.captures(&info) {
                        let names = names_caps["names"].to_string();
                        return IrcMessage::NamesList(names);
                    }
                }
                "366" => {
                    let re_366 = Regex::new(r"^(?P<channel>#[^ ]+)\s:(?P<info>.*)$").unwrap();
                    if let Some(_) = re_366.captures(&info) {
                        return IrcMessage::EndOfNames;
                    }
                }
                _ => {
                    return IrcMessage::Numeric { code, user, info };
                }
            }
        }

        static PRIVMSG_RE: &str =
            r"^:(?P<username>[^!]+)![^@]+@[^ ]+\sPRIVMSG\s(?P<channel>#[^ ]+)\s:(?P<message>.*)$";
        let privmsg = Regex::new(PRIVMSG_RE).unwrap();

        if let Some(caps) = privmsg.captures(&line) {
            let username = caps["username"].to_string();
            let message = caps["message"].to_string();
            return IrcMessage::PrivMsg { username, message };
        }

        static JOIN_RE: &str = r"^:(?P<username>[^!]+)![^@]+@[^ ]+\sJOIN\s(?P<channel>#[^ ]+)$";
        let join_re = Regex::new(JOIN_RE).unwrap();

        if let Some(caps) = join_re.captures(&line) {
            let username = caps["username"].to_string();
            return IrcMessage::Join(username);
        }

        IrcMessage::Unknown(line.to_string())
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
            if let Err(e) = Self::irc_loop(&user_login, &oauth_token, messages_ref).await {
                eprintln!("[start_irc_listener] Error in IRC loop: {e}");
            }
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
        let refresh_interval = Duration::from_secs(30 * 60);

        let mut bg_lock = self.background_task.lock().await;
        let handle = tokio::spawn(async move {
            let client = reqwest::Client::new();

            loop {
                sleep(refresh_interval).await;
                let current_refresh_token = {
                    let tokens = tokens_ref.lock().await;
                    tokens.refresh_token.clone()
                };
                let result = client
                    .post("https://id.twitch.tv/oauth2/token")
                    .form(&[
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("grant_type", "refresh_token"),
                        ("refresh_token", current_refresh_token.unwrap().as_str()),
                    ])
                    .send()
                    .await
                    .and_then(|r| r.error_for_status())
                    .unwrap();
                let json = result.json::<TwitchUserTokens>().await;

                match json {
                    Ok(new_tokens) => {
                        let mut tokens_lock = tokens_ref.lock().await;
                        *tokens_lock = new_tokens;
                        println!("Successfully refreshed Twitch token!");
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh token: {e}");
                    }
                }
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
                    let blob = if message.starts_with("!imgfloat") {
                        let url = message["!imgfloat ".len()..].trim().to_string();
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
