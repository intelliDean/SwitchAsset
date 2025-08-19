AuthentiChain: Blockchain Asset Registry
AuthentiChain is a Rust-based application that interacts with a SwitchAssets smart contract on the Base Sepolia testnet (Layer 2 on Ethereum) to track asset registration and ownership transfers. It includes a PostgreSQL database for caching events, an Axum-based REST API for querying data, and a web frontend for visualizing assets and analytics. This README covers Parts 1 to 3 of the interview assessment:

Part 1: Solidity smart contract for asset registry and transfer.
Part 2: Backend with event listener and REST API.
Part 3: Frontend with analytics and charts.

Table of Contents

Overview
Features
Architecture
Prerequisites
Setup Instructions
Running the Project
API Endpoints
Frontend
Database Schema
Smart Contract
Troubleshooting
Contributing
License

Overview
AuthentiChain monitors the SwitchAssets smart contract (deployed at 0xb91f90fc5c8125226486417db014eaa21f7b27a0 on Base Sepolia) for AssetRegistered and OwnershipTransferred events. It caches these events in a PostgreSQL database for efficient querying, provides a REST API for accessing asset and transfer data, and displays analytics (e.g., total assets, transfers, top owners) on a web frontend. Key goals:

Efficient Queries: Indexed database ensures sub-second API responses for 500+ events.
No Blockchain Rescans: Events are cached in PostgreSQL, avoiding repeated eth_getLogs calls.
Real-Time Updates: Streams new events and updates analytics dynamically.

Features

Smart Contract: SwitchAssets.sol manages asset registration and transfers, emitting AssetRegistered (asset ID, owner) and OwnershipTransferred (asset ID, old/new owner) events.
Event Listener: Rust-based (src/event_listener.rs, version ID: e7f8a9b0-1234-5678-9012-3456789012ef) fetches historical events (last 1,000 blocks in 499-block chunks) and streams new events.
REST API: Axum server (src/app_route.rs, version ID: a1b2c3d4-5678-9012-3456-7890123456ef) provides endpoints for assets, transfers, and analytics.
Database: PostgreSQL with indexed assets and transfers tables (src/schema.rs, version ID: a0b1c2d3-4567-8901-2345-678901234567).
Frontend: HTML/JavaScript interface (static/index.html, version ID: d4e5f6a7-8901-2345-6789-0123456789ef) with Chart.js for transfer visualizations and Tailwind CSS for styling.
Analytics: Generates analytics.json (src/analytics.rs, version ID: c4d5e6f7-8901-2345-abcd-6789012345cd) with metrics like total_assets and top_owners.
Static Assets: Serves logo (static/switch.png) for branding.

Architecture
graph TD
A[User] -->|Interact| B[Web Frontend]
B -->|HTTP Requests| C[REST API (Axum)]
C -->|Query| D[PostgreSQL Database]
D -->|Cached Data| E[Asset Records]
D -->|Cached Data| F[Transfer Records]
C -->|Contract Calls| G[SwitchAssets Smart Contract]
G -->|Deployed on| H[Base Sepolia Testnet]
I[Event Listener] -->|Listen for Events| H
I -->|Store Events| D
I -->|Generate| J[Analytics (analytics.json)]
B -->|Display| J
B -->|Display| K[Charts (Chart.js)]
C -->|Serve| L[Static Files (switch.png)]

    subgraph Off-Chain Services
        C
        D
        I
        J
        L
    end

    subgraph Blockchain
        G
        H
    end

    subgraph Frontend
        B
        K
    end


Frontend: static/index.html displays assets, transfers, and charts.
API: src/app_route.rs serves cached data via /assets, /transfers/:asset_id, etc.
Database: src/schema.rs stores indexed tables.
Event Listener: src/event_listener.rs caches blockchain events.
Smart Contract: SwitchAssets.sol on Base Sepolia.
Analytics: src/analytics.rs generates analytics.json.

Prerequisites

Rust: Stable toolchain (e.g., 1.80.0).
PostgreSQL: Version 14+.
Node.js: For frontend dependencies (Chart.js, Tailwind CSS).
Base Sepolia Access: Provider URL (e.g., Infura, Alchemy) and private key.
Dependencies (Cargo.toml, version ID: b2c3d4e5-6789-0123-4567-8901234567ef):[dependencies]
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



Setup Instructions

Clone the Repository:
git clone <repository-url>
cd authenti_chain


Install Rust:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update


Install PostgreSQL:

Ubuntu:sudo apt update
sudo apt install postgresql postgresql-contrib


macOS:brew install postgresql


Start PostgreSQL:sudo service postgresql start




Set Up the Database:

Create a database:psql -c "CREATE DATABASE switch_assets;"


Install diesel CLI:cargo install diesel_cli --no-default-features --features postgres


Run schema migrations:diesel setup --database-url postgres://localhost/switch_assets
diesel migration run


Create indexes (src/schema.rs, version ID: a0b1c2d3-4567-8901-2345-678901234567):psql -d switch_assets -c "CREATE UNIQUE INDEX idx_assets_asset_id ON assets (asset_id);"
psql -d switch_assets -c "CREATE INDEX idx_assets_owner ON assets (owner);"
psql -d switch_assets -c "CREATE INDEX idx_assets_registered_at ON assets (registered_at);"
psql -d switch_assets -c "CREATE INDEX idx_transfers_asset_id ON transfers (asset_id);"
psql -d switch_assets -c "CREATE INDEX idx_transfers_timestamp ON transfers (timestamp);"
psql -d switch_assets -c "CREATE UNIQUE INDEX idx_transfers_asset_id_timestamp ON transfers (asset_id, timestamp);"




Configure Environment Variables:

Create .env file:echo "DATABASE_URL=postgres://localhost/switch_assets" > .env
echo "BASE_URL=https://sepolia.base.org" >> .env
echo "PRIVATE_KEY=<your-wallet-private-key>" >> .env
echo "CONTRACT_ADDRESS=0xb91f90fc5c8125226486417db014eaa21f7b27a0" >> .env


Replace <your-wallet-private-key> with your Base Sepolia wallet private key. Use a testnet wallet for safety.
Note: The contract address is provided (0xb91f90fc5c8125226486417db014eaa21f7b27a0).


Install Frontend Dependencies:

Install Node.js:curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs


Install Tailwind CSS and Chart.js:npm install -D tailwindcss
npm install chart.js
npx tailwindcss init


Ensure static/switch.png exists in the static/ directory (or switch.svg for version ID: f2a3b4c5-6789-0123-4567-8901234567ef).


Verify Smart Contract:

The SwitchAssets contract is deployed at 0xb91f90fc5c8125226486417db014eaa21f7b27a0 on Base Sepolia.
Check the contract on Base Sepolia Explorer.
If redeploying, use Hardhat or Foundry and update CONTRACT_ADDRESS in .env.



Running the Project

Build and Run the Backend:
cargo run


Expect logs (as of August 19, 2025, 7:54 PM WAT):Server running on 127.0.0.1:8080
Swagger UI available at http://127.0.0.1:8080/swagger-ui/index.html#/
Querying historical events from block 29926654 to 29927153 (range: 500)
Querying historical events from block 29927154 to 29927653 (range: 500)
Starting event stream from block 29927655


If events are detected, expect:📦 Asset Registered: ID = 0x..., Owner = 0x...
🔄 Ownership Transferred: Asset ID = 0x..., Old Owner = 0x..., New Owner = 0x...




Access the Frontend:

Open http://127.0.0.1:8080/chart in a browser.
Verify:
Logo (switch.png) displays in the header.
Analytics (total assets, transfers, top owners) and charts (via Chart.js) load.
Refresh button updates data.




Test API Endpoints:

Get all assets:curl http://127.0.0.1:8080/assets


Get transfers for an asset:curl http://127.0.0.1:8080/transfers/0x1234567890abcdef


Get assets by owner:curl http://127.0.0.1:8080/assets/owner/0x_owner_address


Check analytics:curl http://127.0.0.1:8080/analytics


Swagger UI: http://127.0.0.1:8080/swagger-ui/index.html#/


Trigger a Transfer Event:

Register an asset (if none exist):curl -X POST http://127.0.0.1:8080/contract/register \
-H "Content-Type: application/json" \
-d '{"description": "Luxury Watch"}'


Transfer an asset:curl -X POST http://127.0.0.1:8080/contract/transfer \
-H "Content-Type: application/json" \
-d '{"asset_id": "0x1234567890abcdef", "to": "0x_new_owner2"}'


Expect log:🔄 Ownership Transferred: Asset ID = 0x1234567890abcdef, Old Owner = ..., New Owner = 0x_new_owner2




Verify Database:
psql -d switch_assets -c "SELECT * FROM assets;"
psql -d switch_assets -c "SELECT * FROM transfers;"


Confirm assets and transfers tables reflect events.


Check Analytics:
cat analytics.json


Example:{
"total_assets": 7,
"total_transfers": 5,
"top_owners": [
{"owner": "0xc6fb3fe7c22220862a1b403e5fece8f13bcb61ce", "transfer_count": 3},
{"owner": "0x_new_owner", "transfer_count": 1}
]
}





API Endpoints

GET /assets: List all assets.
GET /assets/:id: Get asset by ID.
GET /assets/owner/:address: Get assets by owner.
GET /transfers/:asset_id: Get transfer history for an asset.
GET /transfers_by_date: Get transfers grouped by date.
GET /analytics: Get analytics (total assets, transfers, top owners).
POST /contract/register: Register a new asset (JSON: { "description": "..." }).
POST /contract/transfer: Transfer asset ownership (JSON: { "asset_id": "0x...", "to": "0x..." }).
GET /contract/get_all_assets: Get all assets from the contract.
GET /contract/get_my_assets: Get caller’s assets from the contract.
GET /static/*: Serve static files (e.g., /static/switch.png).

Performance: Indexed database ensures <1s responses for 500+ events.
Frontend

URL: http://127.0.0.1:8080/chart
Features:
Displays logo (static/switch.png, version ID: d4e5f6a7-8901-2345-6789-0123456789ef).
Shows asset list, transfer history, and analytics.
Visualizes transfer counts with Chart.js.


Files:
static/index.html
static/switch.png (or switch.svg for version ID: f2a3b4c5-6789-0123-4567-8901234567ef).



Database Schema

Assets Table (src/schema.rs, version ID: a0b1c2d3-4567-8901-2345-678901234567):
asset_id: Text (primary key, indexed).
owner: Text (indexed).
description: Text.
registered_at: Int8 (indexed).


Transfers Table:
id: Int4 (auto-incremented).
asset_id: Text (indexed).
old_owner: Text.
new_owner: Text.
timestamp: Int8 (indexed).


Indexes:
idx_assets_asset_id: Unique index on asset_id.
idx_assets_owner: Index on owner.
idx_assets_registered_at: Index on registered_at.
idx_transfers_asset_id: Index on asset_id.
idx_transfers_timestamp: Index on timestamp.
idx_transfers_asset_id_timestamp: Unique composite index.



Smart Contract

File: SwitchAssets.sol
Address: 0xb91f90fc5c8125226486417db014eaa21f7b27a0 (Base Sepolia).
Features:
registerAsset(description): Registers an asset with a unique ID (keccak256 hash), owner, description, and timestamp.
transferAsset(assetId, newOwner): Transfers asset ownership, with checks for ownership and zero addresses.
getAsset(id): Returns asset details.
getAllAssets(): Returns all registered assets.
getMyAssets(): Returns caller’s assets.


Events:
AssetRegistered(bytes32 assetId, address assetOwner)
OwnershipTransferred(bytes32 assetId, address oldOwner, address newOwner)


Modifiers: addressZeroCheck prevents zero-address interactions.
Explorer: Base Sepolia Explorer.

Troubleshooting

No Events in Logs:
Verify CONTRACT_ADDRESS in .env matches 0xb91f90fc5c8125226486417db014eaa21f7b27a0.
Check block range (e.g., blocks 29926654–29927653 may have no events):// src/event_listener.rs
let from_block = latest_block.saturating_sub(100);
let chunk_size = 50;


Test contract: curl http://127.0.0.1:8080/contract/get_all_assets.


Block Range Errors:
If eth_getLogs fails, reduce chunk_size to 200 in event_listener.rs:let chunk_size = 200;


Verify BASE_URL (e.g., https://sepolia.base.org).


Slow API Responses:
Confirm indexes: psql -d switch_assets -c "\d assets;".
Run EXPLAIN ANALYZE:psql -d switch_assets -c "EXPLAIN ANALYZE SELECT * FROM transfers WHERE asset_id = '0x1';"




Event Stream Errors:
Test provider:curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' $BASE_URL


Ensure PRIVATE_KEY is valid for Base Sepolia.


Logo Not Showing:
Test: curl http://127.0.0.1:8080/static/switch.png.
Verify app_route.rs includes:.nest_service("/static", ServeDir::new("static"))





Contributing

Submit issues or pull requests to the repository.
Run cargo fmt and cargo clippy before submitting.
Test API performance with 500+ events:psql -d switch_assets -c "INSERT INTO transfers (asset_id, old_owner, new_owner, timestamp) SELECT '0x' || generate_series(1, 500), '0x_old_owner', '0x_new_owner', 1695062400;"
time curl http://127.0.0.1:8080/transfers/0x1



License
MIT License. See LICENSE file for details.