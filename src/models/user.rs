use diesel::prelude::*;
use uuid::Uuid;

use super::ChannelAdmin;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::models::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: String,
    pub twitch_username: String,
}

impl User {
    pub fn new(twitch_username: String) -> Self {
        let id = Uuid::new_v4().into();
        Self {
            id,
            twitch_username,
        }
    }

    pub fn into_channel_admin(self, broadcaster: User) -> ChannelAdmin {
        ChannelAdmin::new(
            self.twitch_username.clone(),
            broadcaster.twitch_username.clone(),
        )
    }
}
