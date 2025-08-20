CREATE TABLE IF NOT EXISTS assets
(
    asset_id      TEXT PRIMARY KEY,
    owner         TEXT   NOT NULL,
    description   TEXT   NOT NULL,
    registered_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS transfers
(
    id        SERIAL PRIMARY KEY,
    asset_id  TEXT   NOT NULL REFERENCES assets (asset_id),
    old_owner TEXT   NOT NULL,
    new_owner TEXT   NOT NULL,
    timestamp BIGINT NOT NULL,
    txn_hash  TEXT   NOT NULL
);