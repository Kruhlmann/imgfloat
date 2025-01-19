use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::models::schema::channel_admin)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ChannelAdmin {
    pub id: String,
    pub admin_username: String,
    pub broadcaster_username: String,
}

impl ChannelAdmin {
    pub fn new(admin_username: String, broadcaster_username: String) -> Self {
        let id = Uuid::new_v4().into();
        Self {
            id,
            admin_username,
            broadcaster_username,
        }
    }
}
