use chrono::NaiveDateTime;
#[cfg(feature = "server")]
use diesel::prelude::*;
use std::time::Duration;
use uuid::Uuid;

use super::User;

#[derive(Debug, PartialEq)]
#[cfg_attr(
    feature = "server",
    derive(Identifiable, Queryable, Selectable, Insertable)
)]
#[cfg_attr(feature = "server", diesel(table_name = crate::models::schema::sessions))]
#[cfg_attr(feature = "server", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
#[cfg_attr(feature = "server", diesel(primary_key(username)))]
pub struct Session {
    pub username: String,
    pub token: String,
    pub expires: NaiveDateTime,
}

impl Session {
    pub fn new(user: impl Into<User>) -> Self {
        Self {
            username: user.into().username,
            token: Uuid::new_v4().into(),
            expires: NaiveDateTime::default() + Duration::from_days(30),
        }
    }
}

impl From<User> for Session {
    fn from(value: User) -> Self {
        Self {
            username: value.username.clone(),
            token: Uuid::new_v4().into(),
            expires: NaiveDateTime::default() + Duration::from_days(30),
        }
    }
}
