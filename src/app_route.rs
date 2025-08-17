use std::env;
use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use ethabi::ethereum_types::Address;
use ethers::contract::abigen;
use ethers::prelude::{Http, Provider};
use eyre::Report;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::app_state::AppState;
use crate::swagger::ApiDoc;
use crate::handlers::assets::get_all_assets;
use crate::handlers::transfer::{get_assets_by_owner, get_transfers_by_asset};

abigen!(
    SwitchAssets,
    r#"[
        event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner)
        event OwnershipTransferred(bytes32 indexed assetId, address indexed oldOwner, address indexed newOwner)
        function getAsset(bytes32) view returns (bytes32, address, string, uint256)
        function getMyAssets() view returns ((bytes32, address, string, uint256)[])
    ]"#
);

pub fn state_init() -> eyre::Result<AppState, Report> {
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

    let state = AppState {
        db_pool: pool,
        contract,
    };
    Ok(state)
}

pub fn app_router(state: AppState) -> Router {
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/assets", get(get_all_assets))
        .route("/transfers/{asset_id}", get(get_transfers_by_asset))
        .route("/assets/owner/{address}", get(get_assets_by_owner))
        .with_state(state);
    app
}
