use diesel::prelude::*;
use uuid::Uuid;

use super::ChannelAdmin;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::models::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: String,
    pub twitch_username: String,
}

impl User {
    pub fn new(twitch_username: &str) -> Self {
        let id = Uuid::new_v4().into();
        tracing::info!(?id, ?twitch_username, "created channel admin");
        Self {
            id,
            twitch_username: twitch_username.to_string(),
        }
    }

    pub fn into_channel_admin(self, broadcaster: User) -> ChannelAdmin {
        tracing::info!(user = ?self, ?broadcaster, "turned user into channel admin");
        ChannelAdmin::new(
            self.twitch_username.clone(),
            broadcaster.twitch_username.clone(),
        )
    }
}
