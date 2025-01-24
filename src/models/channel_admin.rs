use diesel::prelude::*;

use super::User;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::models::schema::channel_admins)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
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
