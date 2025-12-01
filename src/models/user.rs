#[cfg(feature = "server")]
use diesel::prelude::*;

use crate::twitch::TwitchUser;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "server",
    derive(Identifiable, Queryable, Selectable, Insertable)
)]
#[cfg_attr(feature = "server", diesel(table_name = crate::models::schema::users))]
#[cfg_attr(feature = "server", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
#[cfg_attr(feature = "server", diesel(primary_key(username)))]
pub struct User {
    pub username: String,
}

impl User {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.to_string(),
        }
    }
}

impl From<TwitchUser> for User {
    fn from(value: TwitchUser) -> Self {
        Self {
            username: value.login,
        }
    }
}

impl From<&TwitchUser> for User {
    fn from(value: &TwitchUser) -> Self {
        Self {
            username: value.login.clone(),
        }
    }
}
