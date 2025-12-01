pub mod auth_callback_query;
pub mod credentials;
pub mod response;
pub mod user;
pub mod user_tokens;

pub use auth_callback_query::AuthCallbackFailureQuery;
pub use auth_callback_query::AuthCallbackQuery;
pub use auth_callback_query::AuthCallbackSuccessQuery;
pub use credentials::TwitchCredentials;
pub use response::TwitchApiResponse;
pub use user::TwitchUser;
pub use user_tokens::TwitchUserTokens;

#[cfg(feature = "server")]
pub mod authenticator;

#[cfg(feature = "server")]
pub use authenticator::TwitchAuthenticator;
#[cfg(feature = "server")]
pub use authenticator::TwitchAuthenticatorError;
#[cfg(feature = "server")]
pub use authenticator::TwitchHttpAuthenticator;
