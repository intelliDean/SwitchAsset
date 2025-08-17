// @generated automatically by Diesel CLI.

diesel::table! {
    User (id) {
        id -> Text,
        walletAddress -> Text,
        username -> Text,
        registeredAt -> Timestamp,
        createdAt -> Timestamp,
        updatedAt -> Timestamp,
    }
}

diesel::table! {
    _prisma_migrations (id) {
        #[max_length = 36]
        id -> Varchar,
        #[max_length = 64]
        checksum -> Varchar,
        finished_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        migration_name -> Varchar,
        logs -> Nullable<Text>,
        rolled_back_at -> Nullable<Timestamptz>,
        started_at -> Timestamptz,
        applied_steps_count -> Int4,
    }
}

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

diesel::allow_tables_to_appear_in_same_query!(
    User,
    _prisma_migrations,
    assets,
    transfers,
);
