// @generated automatically by Diesel CLI.

diesel::table! {
    assets (local_filename) {
        local_filename -> Text,
        original_filename -> Text,
        checksum -> Text,
        content_type -> Text,
        username -> Text,
    }
}

diesel::table! {
    channel_admins (username, broadcaster_username) {
        username -> Text,
        broadcaster_username -> Text,
    }
}

diesel::table! {
    user_settings (username) {
        username -> Text,
        background_opacity -> Float,
        fps_target -> Integer,
    }
}

diesel::table! {
    users (username) {
        username -> Text,
    }
}

diesel::joinable!(assets -> users (username));
diesel::joinable!(user_settings -> users (username));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    channel_admins,
    user_settings,
    users,
);
