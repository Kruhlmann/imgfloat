// @generated automatically by Diesel CLI.

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

diesel::allow_tables_to_appear_in_same_query!(
    channel_admins,
    users,
);
