use diesel::prelude::*;

use crate::twitch::TwitchUser;

#[derive(Debug, PartialEq, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::models::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
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
