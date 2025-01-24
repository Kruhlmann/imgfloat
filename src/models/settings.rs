use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Debug, Eq, PartialEq, serde::Serialize)]
#[diesel(table_name = crate::models::schema::user_settings)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserSettings {
    pub username: String,
    pub background_opacity: i32,
    pub fps_target: i32,
}
