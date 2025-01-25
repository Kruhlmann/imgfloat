use diesel::prelude::*;

use super::User;

#[derive(
    Queryable, Selectable, Insertable, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[diesel(table_name = crate::models::schema::user_settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserSettings {
    pub username: String,
    pub background_opacity: i32,
    pub fps_target: i32,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
pub struct UnownedUserSettings {
    pub background_opacity: u8,
    pub fps_target: u16,
}

impl UnownedUserSettings {
    pub fn with_owner(self, owner: &User) -> UserSettings {
        UserSettings {
            username: owner.username.clone(),
            background_opacity: self.background_opacity.into(),
            fps_target: self.fps_target.into(),
        }
    }
}

impl Default for UnownedUserSettings {
    fn default() -> Self {
        UnownedUserSettings {
            background_opacity: 40,
            fps_target: 60,
        }
    }
}
