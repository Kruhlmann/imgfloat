use diesel::SqliteConnection;

use crate::models::{Asset, ChannelAdmin, User};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub struct SqliteDbService {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteDbService {
    pub fn new(database_url: &str) -> Result<Self, r2d2::Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(16)
            .build(manager)
            .inspect_err(|error| tracing::error!(?error, "db connection failed"))?;
        Ok(Self { pool })
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        use crate::models::schema::users::dsl::*;
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))
            .ok()?;
        users
            .filter(twitch_username.eq(username))
            .first::<User>(&mut conn)
            .optional()
            .inspect_err(|error| tracing::error!(?error, "get user"))
            .ok()
            .flatten()
    }

    pub fn create_user(&self, user: &User) -> Result<User, Box<dyn std::error::Error>> {
        use crate::models::schema::users::dsl::*;
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        Ok(diesel::insert_into(users)
            .values(user)
            .get_result::<User>(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "create user"))?)
    }

    pub fn create_channel_admin(
        &self,
        channel_admin: &ChannelAdmin,
    ) -> Result<ChannelAdmin, Box<dyn std::error::Error>> {
        use crate::models::schema::channel_admins::dsl::*;
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        Ok(diesel::insert_into(channel_admins)
            .values(channel_admin)
            .get_result::<ChannelAdmin>(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "create channel admin"))?)
    }

    pub fn create_asset(&self, asset: &Asset) -> Result<Asset, Box<dyn std::error::Error>> {
        use crate::models::schema::assets::dsl::*;
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        Ok(diesel::insert_into(assets)
            .values(asset)
            .get_result::<Asset>(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "create asset"))?)
    }

    pub fn get_broadcaster_assets(
        &self,
        broadcaster: &User,
    ) -> Result<Vec<Asset>, Box<dyn std::error::Error>> {
        use crate::models::schema::assets::dsl::*;
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;

        Ok(assets
            .filter(broadcaster_username.eq(&broadcaster.twitch_username))
            .select(Asset::as_select())
            .load(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "get broadcaster assets"))?)
    }
}
