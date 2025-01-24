use diesel::SqliteConnection;

use crate::models::{Asset, ChannelAdmin, User, UserSettings};
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
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))
            .ok()?;
        crate::models::schema::users::dsl::users
            .filter(crate::models::schema::users::dsl::username.eq(username))
            .first::<User>(&mut conn)
            .optional()
            .inspect_err(|error| tracing::error!(?error, "get user"))
            .ok()
            .flatten()
    }

    pub fn create_user(&self, user: &User) -> Result<User, Box<dyn std::error::Error>> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        let user = diesel::insert_into(crate::models::schema::users::dsl::users)
            .values(user)
            .get_result::<User>(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "create user"))?;
        Ok(user)
    }

    pub fn get_user_settings(&self, user: &User) -> Option<UserSettings> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))
            .ok()?;
        crate::models::schema::user_settings::dsl::user_settings
            .filter(crate::models::schema::user_settings::dsl::username.eq(&user.username))
            .first::<UserSettings>(&mut conn)
            .optional()
            .inspect_err(|error| tracing::error!(?error, "get user settings"))
            .ok()
            .flatten()
    }

    pub fn update_user_settings(
        &self,
        settings: &UserSettings,
    ) -> Result<UserSettings, Box<dyn std::error::Error>> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        let new_settings =
            diesel::insert_into(crate::models::schema::user_settings::dsl::user_settings)
                .values(settings)
                .get_result::<UserSettings>(&mut conn)
                .inspect_err(|error| tracing::error!(?error, "update user settings"))?;
        Ok(new_settings)
    }

    pub fn create_user_settings(
        &self,
        settings: &UserSettings,
    ) -> Result<UserSettings, Box<dyn std::error::Error>> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        let new_settings =
            diesel::insert_into(crate::models::schema::user_settings::dsl::user_settings)
                .values(settings)
                .get_result::<UserSettings>(&mut conn)
                .inspect_err(|error| tracing::error!(?error, "create user settings"))?;
        Ok(new_settings)
    }

    pub fn create_channel_admin(
        &self,
        channel_admin: &ChannelAdmin,
    ) -> Result<ChannelAdmin, Box<dyn std::error::Error>> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        let new_channel_admin =
            diesel::insert_into(crate::models::schema::channel_admins::dsl::channel_admins)
                .values(channel_admin)
                .get_result::<ChannelAdmin>(&mut conn)
                .inspect_err(|error| tracing::error!(?error, "create channel admin"))?;
        Ok(new_channel_admin)
    }

    pub fn get_asset(&self, filename: &str) -> Option<Asset> {
        tracing::debug!(?filename, "looking for asset");
        let mut conn = self.pool.get().ok()?;
        match crate::models::schema::assets::dsl::assets
            .filter(crate::models::schema::assets::dsl::local_filename.eq(filename))
            .first::<Asset>(&mut conn)
            .optional()
        {
            Ok(asset) => asset,
            Err(e) => {
                tracing::error!(?e, "Error fetching asset from DB");
                None
            }
        }
    }

    pub fn create_asset(&self, asset: &Asset) -> Result<Asset, Box<dyn std::error::Error>> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        let new_asset = diesel::insert_into(crate::models::schema::assets::dsl::assets)
            .values(asset)
            .get_result::<Asset>(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "create asset"))?;
        Ok(new_asset)
    }

    pub fn get_broadcaster_assets(
        &self,
        broadcaster: &User,
    ) -> Result<Vec<Asset>, Box<dyn std::error::Error>> {
        let mut conn = self
            .pool
            .get()
            .inspect_err(|error| tracing::error!(?error, "unable to get db connection"))?;
        let broadcaster_assets = crate::models::schema::assets::dsl::assets
            .filter(crate::models::schema::assets::dsl::username.eq(&broadcaster.username))
            .select(Asset::as_select())
            .load(&mut conn)
            .inspect_err(|error| tracing::error!(?error, "get broadcaster assets"))?;
        Ok(broadcaster_assets)
    }
}
