use diesel::table;

table! {
    assets (asset_id) {
        asset_id -> Text,
        owner -> Text,
        description -> Text,
        registered_at -> BigInt,
    }
}

table! {
    transfers (id) {
        id -> Int4,
        asset_id -> Text,
        old_owner -> Text,
        new_owner -> Text,
        timestamp -> BigInt,
    }
}
