# Switch Asset: Blockchain Asset Registry

Switch Asset is a simple on-chain Asset Registry to track asset registration and ownership transfers. The smart contract is deployed on the Base Sepolia testnet (Layer 2 on Ethereum). It includes an Axum-based REST API for querying data, a PostgreSQL database for caching events, and a web frontend for visualizing assets and analytics.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Setup Instructions](#setup-instructions)
- [Running the Project](#running-the-project)
- [API Endpoints](#api-endpoints)
- [Frontend](#frontend)
- [Database Schema](#database-schema)
- [Smart Contract](#smart-contract)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)

## Overview


SwitchAssets smart contract (deployed at [0x3897196da6a4f2219ED4F183AFA3A10C8C227f23](https://sepolia.basescan.org/address/0x3897196da6a4f2219ED4F183AFA3A10C8C227f23#code) on Base Sepolia) for `AssetRegistered` and `OwnershipTransferred` events. It caches these events in a PostgreSQL database for efficient querying, provides a REST API for accessing asset and transfer data, and displays analytics (e.g., total assets, transfers, top owners) on a web frontend.

**Key goals:**
- Efficient Queries: Indexed database ensures sub-second API responses for 500+ events
- No Blockchain Rescans: Events are cached in PostgreSQL, avoiding repeated eth_getLogs calls
- Real-Time Updates: Streams new events and updates analytics dynamically
## Features

- **Smart Contract**: SwitchAssets.sol manages asset registration and transfers, emitting events
- **Event Listener**: Rust-based listener fetches historical events and streams new events
- **REST API**: Axum server provides endpoints for assets, transfers, and analytics
- **Database**: PostgreSQL with indexed assets and transfers tables
- **Frontend**: HTML/JavaScript interface with Chart.js for visualizations
- **Analytics**: Generates metrics like total_assets and top_owners
-  **Frontend**: static/index.html displays assets, transfers, and charts.
- **API**: src/app_route.rs serves cached data via /assets, /transfers/:asset_id, etc.
- **Database**: src/schema.rs stores indexed tables. 
- **Event Listener**: src/event_listener.rs caches blockchain events. 
- **Smart Contract**: SwitchAssets.sol on Base Sepolia. 
- **Analytics**: src/analytics.rs generates analytics.json.

## Prerequisites
- Rust: Stable toolchain (e.g., 1.80.0).
- PostgreSQL: Version 14+. 
- Node.js: For frontend dependencies (Chart.js, Tailwind CSS). 
- Base Sepolia Access: Provider URL (e.g., Infura, Alchemy) and private key. 
- Dependencies (Cargo.toml)

```toml
[dependencies]
ethers = { version = "2.0.14", features = ["rustls"] }
tokio = { version = "1.44.2", features = ["full"] }
dotenv = "0.15.0"
anyhow = "1.0.98"
serde_json = "1.0"
hex = "0.4.3"
serde = { version = "1.0.219", features = ["derive"] }
axum = "0.8.3"
utoipa = { version = "5.3.1", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "9.0.1", features = ["axum"] }
ethabi = "18.0.0"
tower-http = { version = "0.6.2", features = ["cors"] }
validator = { version = "0.20.0", features = ["derive"] }
sqlx = "0.8.6"
diesel = { version = "2.2.12", features = ["postgres", "r2d2", "chrono"] }
dotenvy = "0.15.7"
chrono = { version = "0.4.41", features = ["serde"] }
rand = "0.9.2"
eyre = "0.6.12"
ecdsa = "0.16.9"
```

## Setup Instructions
- Clone the Repository:
```bash
git clone https://github.com/intelliDean/SwitchAsset.git
```
- Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```
- Install PostgreSQL:
```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
```
- Start PostgreSQL:
```bash
sudo service postgresql start
```

### Set Up the Database:
- Create a database:
```bash
psql -c "CREATE DATABASE switch_assets;"
```
- Install diesel CLI:
```bash
cargo install diesel_cli --no-default-features --features postgres
```

### Run schema migrations:
```bash
diesel setup --database-url postgres://localhost/switch_assets
diesel migration run
```

- Create indexes (src/schema.rs, version ID: a0b1c2d3-4567-8901-2345-678901234567):
```bash
psql -d switch_assets -c "CREATE UNIQUE INDEX idx_assets_asset_id ON assets (asset_id);"
psql -d switch_assets -c "CREATE INDEX idx_assets_owner ON assets (owner);"
psql -d switch_assets -c "CREATE INDEX idx_assets_registered_at ON assets (registered_at);"
psql -d switch_assets -c "CREATE INDEX idx_transfers_asset_id ON transfers (asset_id);"
psql -d switch_assets -c "CREATE INDEX idx_transfers_timestamp ON transfers (timestamp);"
psql -d switch_assets -c "CREATE UNIQUE INDEX idx_transfers_asset_id_timestamp ON transfers (asset_id, timestamp);"
```

### Configure Environment Variables:

- Create .env file:
```bash
echo "DATABASE_URL=postgres://localhost/switch_assets" > .env
echo "BASE_URL=https://sepolia.base.org" >> .env
echo "PRIVATE_KEY=<your-wallet-private-key>" >> .env
echo "CONTRACT_ADDRESS=0xb91f90fc5c8125226486417db014eaa21f7b27a0" >> .env
```

- Replace <your-wallet-private-key> with your Base Sepolia wallet private key. Use a testnet wallet for safety.
- Note: The contract address is provided [0x3897196da6a4f2219ED4F183AFA3A10C8C227f23](https://sepolia.basescan.org/address/0x3897196da6a4f2219ED4F183AFA3A10C8C227f23#code).

- Install Node.js:
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

- Install Tailwind CSS and Chart.js:
```bash
npm install -D tailwindcss
npm install chart.js
npx tailwindcss init
```

- Verify Smart Contract:
- The SwitchAssets contract is deployed at 0x3897196da6a4f2219ED4F183AFA3A10C8C227f23 on Base Sepolia.
- Check the contract on Base Sepolia Explorer.
- If redeploying, use Hardhat or Foundry and update CONTRACT_ADDRESS in .env.

## Running the Project

### Build and Run the Backend:
```bash
cargo run
```
- Expect logs (as of August 19, 2025, 7:54 PM WAT):
```
Server running on 127.0.0.1:8080
Swagger UI available at http://127.0.0.1:8080/swagger-ui/index.html#/
Querying historical events from block 29926654 to 29927153 (range: 500)
Querying historical events from block 29927154 to 29927653 (range: 500)
Starting event stream from block 29927655
```

- If events are detected, expect:
```
📦 Asset Registered: ID = 0x..., Owner = 0x...
🔄 Ownership Transferred: Asset ID = 0x..., Old Owner = 0x..., New Owner = 0x...
```


### Access the Frontend:
- Open http://127.0.0.1:8080/chart in a browser.
- Verify:

  - Analytics (total assets, transfers, top owners) and charts (via Chart.js) load. 
  - Refresh button updates data.

### Test API Endpoints:

- Get all assets:
```bash
curl http://127.0.0.1:8080/assets
```

- Get transfers for an asset:
```bash
curl http://127.0.0.1:8080/transfers/0x1234567890abcdef
```

- Get assets by owner:
```bash
curl http://127.0.0.1:8080/assets/owner/0x_owner_address
```

- Check analytics:
```bash
curl http://127.0.0.1:8080/analytics
```
- You can also access the API Endpoints via Swagger UI (Recommended)
    - Swagger UI: http://127.0.0.1:8080/swagger-ui/index.html#/

### Trigger a Transfer Event:

- Register an asset (if none exist):
```bash
curl -X POST http://127.0.0.1:8080/contract/register \
-H "Content-Type: application/json" \
-d '{"description": "Luxury Watch"}'
```

- Transfer an asset:
```bash
curl -X POST http://127.0.0.1:8080/contract/transfer \
-H "Content-Type: application/json" \
-d '{"asset_id": "0x1234567890abcdef", "to": "0x_new_owner2"}'
```


### Verify Database:
```bash
psql -d switch_assets -c "SELECT * FROM assets;"
psql -d switch_assets -c "SELECT * FROM transfers;"
```

- Confirm assets and transfers tables reflect events.

- Check Analytics:
```bash
cat analytics.json
```
- Example:
```json
{
  "total_assets": 7,
  "total_transfers": 5,
  "top_owners": [
    {
      "owner": "0xc6fb3fe7c22220862a1b403e5fece8f13bcb61ce", 
      "transfer_count": 3
    },
    {
      "owner": "0x_new_owner", 
      "transfer_count": 1
    }
  ]
}
```

### API Endpoints
- GET /assets: List all assets.
- GET /assets/:id: Get asset by ID.
- GET /assets/owner/:address: Get assets by owner.
- GET /transfers/:asset_id: Get transfer history for an asset.
- GET /transfers_by_date: Get transfers grouped by date.
- GET /analytics: Get analytics (total assets, transfers, top owners).
- POST /contract/register: Register a new asset (JSON: { "description": "..." }).
- POST /contract/transfer: Transfer asset ownership (JSON: { "asset_id": "0x...", "to": "0x..." }).
- GET /contract/get_all_assets: Get all assets from the contract.
- GET /contract/get_my_assets: Get caller’s assets from the contract.
- GET /static/*: Serve static files (e.g., /static/switch.png).
- Performance: Indexed database ensures <1s responses for 500+ events.

### Frontend
- URL: http://127.0.0.1:8080/chart
- Features:
  - Displays logo (static/switch.png, version ID: d4e5f6a7-8901-2345-6789-0123456789ef). 
  - Shows asset list, transfer history, and analytics. 
  - Visualizes transfer counts with Chart.js.
- Files:
  - file/static/index.html


### Database Schema
- Assets Table (src/schema.rs, version ID: a0b1c2d3-4567-8901-2345-678901234567):
    - asset_id: Text (primary key, indexed).
    - owner: Text (indexed). 
    - description: Text. 
    - registered_at: Int8 (indexed).
- Transfers Table:
  - id: Int4 (auto-incremented).
  - asset_id: Text (indexed).
  - old_owner: Text. 
  - new_owner: Text.
  - timestamp: Int8 (indexed).
  
- Indexes:
  - idx_assets_asset_id: Unique index on asset_id.
  - idx_assets_owner: Index on owner. 
  - idx_assets_registered_at: Index on registered_at. 
  - idx_transfers_asset_id: Index on asset_id. 
  - idx_transfers_timestamp: Index on timestamp.
  - idx_transfers_asset_id_timestamp: Unique composite index.


### Smart Contract
- File: SwitchAssets.sol
- Address: [0x3897196da6a4f2219ED4F183AFA3A10C8C227f23](https://sepolia.basescan.org/address/0x3897196da6a4f2219ED4F183AFA3A10C8C227f23#code) (Base Sepolia).
- Features:
  - registerAsset(description): Registers an asset with a unique ID (keccak256 hash), owner, description, and timestamp. 
  - transferAsset(assetId, newOwner): Transfers asset ownership, with checks for ownership and zero addresses. 
  - getAsset(id): Returns asset details. 
  - getAllAssets(): Returns all registered assets. 
  - getMyAssets(): Returns caller’s assets.
- Events:
  - AssetRegistered(bytes32 assetId, address assetOwner)
  - OwnershipTransferred(bytes32 assetId, address oldOwner, address newOwner)
- Modifiers: addressZeroCheck prevents zero-address interactions. 
- Explorer: Base Sepolia Explorer.


### Contributing
- Submit issues or pull requests to the repository.
- Run cargo fmt and cargo clippy before submitting.
- Test API performance with 500+ events:
```bash
psql -d switch_assets -c "INSERT INTO transfers (asset_id, old_owner, new_owner, timestamp) SELECT '0x' || generate_series(1, 500), '0x_old_owner', '0x_new_owner', 1695062400;"
time curl http://127.0.0.1:8080/transfers/0x1
```