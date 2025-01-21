// @generated automatically by Diesel CLI.

diesel::table! {
    assets (local_filename) {
        local_filename -> Text,
        original_filename -> Text,
        checksum -> Text,
        content_type -> Text,
        broadcaster_username -> Text,
    }
}

diesel::table! {
    channel_admins (id) {
        id -> Text,
        admin_username -> Text,
        broadcaster_username -> Text,
    }
}

diesel::table! {
    users (twitch_username) {
        id -> Text,
        twitch_username -> Text,
    }
}

diesel::joinable!(assets -> users (broadcaster_username));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    channel_admins,
    users,
);
