


CREATE TABLE IF NOT EXISTS assets (
              asset_id TEXT PRIMARY KEY,
              owner TEXT NOT NULL,
              description TEXT NOT NULL,
              registered_at BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS transfers (
                id SERIAL PRIMARY KEY,
                asset_id TEXT NOT NULL REFERENCES assets(asset_id),
                old_owner TEXT NOT NULL,
                new_owner TEXT NOT NULL,
                timestamp BIGINT NOT NULL
);



--
--
-- CREATE TABLE IF NOT EXISTS assets (
--                                       asset_id BYTEA PRIMARY KEY,
--                                       owner BYTEA NOT NULL,
--                                       description TEXT NOT NULL,
--                                       registered_at BIGINT NOT NULL,
--                                       first_tx_hash BYTEA,
--                                       first_block BIGINT,
--                                       first_log_index BIGINT
-- );
--
-- CREATE INDEX IF NOT EXISTS idx_assets_owner ON assets(owner);
--
-- CREATE TABLE IF NOT EXISTS transfers (
--                                          id BIGSERIAL PRIMARY KEY,
--                                          asset_id BYTEA NOT NULL REFERENCES assets(asset_id) ON DELETE CASCADE,
--                                          from_addr BYTEA NOT NULL,
--                                          to_addr BYTEA NOT NULL,
--                                          tx_hash BYTEA NOT NULL,
--                                          block_num BIGINT NOT NULL,
--                                          log_index BIGINT NOT NULL,
--                                          occurred_at BIGINT NOT NULL
-- );
--
-- CREATE INDEX IF NOT EXISTS idx_transfers_asset ON transfers(asset_id);
-- CREATE INDEX IF NOT EXISTS idx_transfers_to ON transfers(to_addr);
-- CREATE INDEX IF NOT EXISTS idx_transfers_from ON transfers(from_addr);