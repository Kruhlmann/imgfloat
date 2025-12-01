#[cfg(feature = "server")]
use diesel::prelude::*;

use super::User;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "server",
    derive(Identifiable, Queryable, Selectable, Insertable,)
)]
#[cfg_attr(feature = "server", diesel(table_name = crate::models::schema::channel_admins))]
#[cfg_attr(feature = "server", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
#[cfg_attr(
    feature = "server",
    diesel(primary_key(username, broadcaster_username))
)]
pub struct ChannelAdmin {
    pub username: String,
    pub broadcaster_username: String,
}

impl ChannelAdmin {
    pub fn new(user: &User, broadcaster: &User) -> Self {
        Self {
            username: user.username.clone(),
            broadcaster_username: broadcaster.username.clone(),
        }
    }
}
