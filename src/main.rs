use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get},
    Json, Router,
};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::Address,
    core::utils::to_checksum, 
};
use eyre::Result;
use serde::{Serialize};
use std::env;
use std::net::{SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use crate::assets::dsl::assets;
use crate::assets::owner;

// Diesel schema
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

// Database models
#[derive(Queryable, Serialize, ToSchema)]
struct Asset {
    asset_id: String,
    owner: String,
    description: String,
    registered_at: i64,
}

#[derive(Queryable, Serialize, ToSchema)]
struct Transfer {
    id: i32,
    asset_id: String,
    old_owner: String,
    new_owner: String,
    timestamp: i64,
}

#[derive(Serialize, ToSchema)]
struct ApiResponse<T> {
    data: T,
}

abigen!(
    SwitchAssets,
    r#"[
        event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner)
        event OwnershipTransferred(bytes32 indexed assetId, address indexed oldOwner, address indexed newOwner)
        function getAsset(bytes32) view returns (bytes32, address, string, uint256)
        function getMyAssets() view returns ((bytes32, address, string, uint256)[])
    ]"#
);

#[derive(Clone)]
struct AppState {
    db_pool: Pool<ConnectionManager<PgConnection>>,
    contract: SwitchAssets<Provider<Http>>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_assets,
        get_transfers_by_asset,
        get_assets_by_owner
    ),
    components(
        schemas(Asset, Transfer, ApiResponse<Vec<Asset>>, ApiResponse<Vec<Transfer>>)
    ),
    tags(
        (name = "SwitchAssets", description = "API for managing blockchain assets")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {

    dotenv().ok();
    let db_url = env::var("DATABASE_URL")?;
    let eth_rpc_url = env::var("BASE_URL")?;
    let contract_address: Address = env::var("CONTRACT_ADDRESS")?.parse()?;

    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .map_err(|e| eyre::eyre!("Failed to create pool: {}", e))?;

    let provider = Provider::<Http>::try_from(eth_rpc_url)?;
    let client = Arc::new(provider);

    let contract = SwitchAssets::new(contract_address, client.clone());

    let state = AppState { db_pool: pool, contract };

    tokio::spawn(listen_for_events(state.clone()));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/assets", get(get_all_assets))
        .route("/transfers/{asset_id}", get(get_transfers_by_asset))
        .route("/assets/owner/{address}", get(get_assets_by_owner))
        .with_state(state);


    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener =  TcpListener::bind(addr).await?;

    println!("Server running on {:?}", addr);
    println!("Swagger UI available at http://127.0.0.1:8080/swagger-ui/index.html#/");

    axum::serve(listener, app).await?;

    Ok(())
}

#[utoipa::path(
    get,
    path = "/assets",
    responses(
        (status = 200, description = "List all registered assets", body = ApiResponse<Vec<Asset>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
async fn get_all_assets(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Asset>>>, StatusCode> {
    use self::assets::dsl::*;
    let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = assets
        .load::<Asset>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}

#[utoipa::path(
    get,
    path = "/transfers/{asset_id}",
    params(
        ("asset_id" = String, Path, description = "Asset ID as hex string")
    ),
    responses(
        (status = 200, description = "List transfers for an asset", body = ApiResponse<Vec<Transfer>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
async fn get_transfers_by_asset(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Transfer>>>, StatusCode> {
    use self::transfers::dsl::*;
    let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = transfers
        .filter(self::transfers::asset_id.eq(asset_id))
        .load::<Transfer>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}

#[utoipa::path(
    get,
    path = "/assets/owner/{address}",
    params(
        ("address" = String, Path, description = "Owner address as hex string")
    ),
    responses(
        (status = 200, description = "List assets owned by address", body = ApiResponse<Vec<Asset>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
async fn get_assets_by_owner(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Asset>>>, StatusCode> {
    use self::assets::dsl::*;
    let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let results = assets
        .filter(self::assets::owner.eq(address))
        .load::<Asset>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}

async fn listen_for_events(state: AppState) -> Result<()> {
    let contract = &state.contract;
    let events = contract.events().from_block(BlockNumber::Latest);

    let mut stream = events.stream().await?;
    while let Some(event) = stream.next().await {
        match event {
            Ok(SwitchAssetsEvents::AssetRegisteredFilter(event)) => {
                let asset_id = format!("0x{}", hex::encode(event.asset_id));
                let owner = to_checksum(&event.asset_owner, None); // Use to_checksum
                let asset = contract
                    .get_asset(event.asset_id)
                    .call()
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                let description = asset.2;
                let registered_at = asset.3.as_u64() as i64;

                let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                diesel::insert_into(assets::table)
                    .values((
                        assets::asset_id.eq(&asset_id),
                        assets::owner.eq(&owner),
                        assets::description.eq(description),
                        assets::registered_at.eq(registered_at),
                    ))
                    .execute(conn)
                    .map_err(|e| {
                        println!("Failed to insert asset: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }).unwrap();
            }
            Ok(SwitchAssetsEvents::OwnershipTransferredFilter(event)) => {
                let asset_id = format!("0x{}", hex::encode(event.asset_id));
                let old_owner = to_checksum(&event.old_owner, None); // Use to_checksum
                let new_owner = to_checksum(&event.new_owner, None); // Use to_checksum
                let timestamp = chrono::Utc::now().timestamp();

                let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR).unwrap();
                diesel::insert_into(transfers::table)
                    .values((
                        transfers::asset_id.eq(&asset_id),
                        transfers::old_owner.eq(&old_owner),
                        transfers::new_owner.eq(&new_owner),
                        transfers::timestamp.eq(timestamp),
                    ))
                    .execute(conn)
                    .map_err(|e| {
                        println!("Failed to insert transfer: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }).unwrap();

                diesel::update(assets::table)
                    .filter(assets::asset_id.eq(&asset_id))
                    .set(assets::owner.eq(&new_owner))
                    .execute(conn)
                    .map_err(|e| {
                        println!("Failed to update asset owner: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }).unwrap();
            }
            Err(e) => {
                println!("Event stream error: {:?}", e);
            }
        }
    }
    Ok(())
}