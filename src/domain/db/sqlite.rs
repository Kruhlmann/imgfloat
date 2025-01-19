use diesel::{
    result::{DatabaseErrorKind, Error},
    Connection, SqliteConnection,
};

use crate::models::{ChannelAdmin, User};
use diesel::prelude::*;

pub struct SqliteDbService {
    connection: SqliteConnection,
}

impl SqliteDbService {
    pub fn new(database_url: &str) -> Self {
        let connection =
            SqliteConnection::establish(database_url).expect("valid database connection");
        Self { connection }
    }

    pub fn get_user(&mut self, username: String) -> Option<User> {
        use crate::models::schema::users::dsl::*;
        users
            .filter(twitch_username.eq(username))
            .first::<User>(&mut self.connection)
            .optional()
            .inspect_err(|error| tracing::error!(?error, "db error on get_user"))
            .ok()
            .flatten()
    }

    pub fn create_user(&mut self, user: User) -> Option<User> {
        use crate::models::schema::users::dsl::*;
        match diesel::insert_into(users)
            .values(&user)
            .get_result::<User>(&mut self.connection)
        {
            Ok(new_user) => Some(new_user),
            Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => None,
            Err(error) => {
                tracing::error!(?error, "db error on create_user");
                None
            }
        }
    }

    pub fn create_channel_admin(&mut self, channel_admin: ChannelAdmin) -> Option<ChannelAdmin> {
        use crate::models::schema::channel_admins::dsl::*;
        match diesel::insert_into(channel_admins)
            .values(&channel_admin)
            .get_result::<User>(&mut self.connection)
        {
            Ok(new_user) => Some(new_user),
            Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => None,
            Err(error) => {
                tracing::error!(?error, "db error on create_user");
                None
            }
        }
    }
}
