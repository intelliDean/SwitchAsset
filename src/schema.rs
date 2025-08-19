// @generated automatically by Diesel CLI.


diesel::table! {
    assets (asset_id) {
        asset_id -> Text,
        owner -> Text,
        description -> Text,
        registered_at -> Int8,
    }
}

diesel::table! {
    transfers (id) {
        id -> Int4,
        asset_id -> Text,
        old_owner -> Text,
        new_owner -> Text,
        timestamp -> Int8,
    }
}

diesel::joinable!(transfers -> assets (asset_id));

