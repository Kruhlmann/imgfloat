#[cfg(feature = "server")]
use diesel::prelude::*;

#[cfg(feature = "server")]
use crate::domain::Percentage;

#[cfg(feature = "server")]
use super::User;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "server",
    derive(AsChangeset, Queryable, Selectable, Insertable,)
)]
#[cfg_attr(feature = "server", diesel(table_name = crate::models::schema::user_settings))]
#[cfg_attr(feature = "server", diesel(check_for_backend(diesel::sqlite::Sqlite)))]
pub struct UserSettings {
    pub username: String,
    pub background_opacity: f32,
    pub fps_target: i32,
}

#[cfg(feature = "server")]
#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
pub struct UnownedUserSettings {
    pub background_opacity: u8,
    pub fps_target: u16,
}

#[cfg(feature = "server")]
impl UnownedUserSettings {
    pub fn validate(self) -> Result<ValidatedUnownedUserSettings, f32> {
        let background_opacity = Percentage::new(self.background_opacity)?;
        Ok(ValidatedUnownedUserSettings {
            background_opacity,
            fps_target: self.fps_target,
        })
    }
}

#[cfg(feature = "server")]
pub struct ValidatedUnownedUserSettings {
    pub background_opacity: Percentage,
    pub fps_target: u16,
}

#[cfg(feature = "server")]
impl ValidatedUnownedUserSettings {
    pub fn with_owner(self, owner: &User) -> UserSettings {
        UserSettings {
            username: owner.username.clone(),
            background_opacity: self.background_opacity.into(),
            fps_target: self.fps_target.into(),
        }
    }
}

#[cfg(feature = "server")]
impl Default for ValidatedUnownedUserSettings {
    fn default() -> Self {
        ValidatedUnownedUserSettings {
            background_opacity: Percentage(40.0),
            fps_target: 60,
        }
    }
}
