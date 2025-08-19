use crate::app_state::AppState;
use crate::handlers::assets::get_all_assets;
use crate::handlers::transfer::{get_assets_by_owner, get_transfers_by_asset};
use crate::swagger::ApiDoc;
use axum::Router;
use axum::routing::{get, post};
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use ethabi::ethereum_types::Address;
use ethers::contract::abigen;
use ethers::middleware::{Middleware, SignerMiddleware};
use ethers::prelude::{Http, LocalWallet, Provider, Signer};
use eyre::Report;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::contract_calls::{get_asset, get_my_assets, register_asset, transfer_asset, get_all_contract_assets};

//abi path
abigen!(
    SwitchAssets,
    "./hh-artifacts/contracts/SwitchAssets.sol/SwitchAssets.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub async fn state_init() -> eyre::Result<AppState, Report> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .map_err(|e| eyre::eyre!("Failed to create pool: {}", e))?;

    let rpc_url = env::var("BASE_URL")?;
    let private_key = env::var("PRIVATE_KEY")?;

    let switch_address: Address = env::var("CONTRACT_ADDRESS")?
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid contract address"))
        .unwrap();

    let provider = Provider::<Http>::try_from(&rpc_url)?.interval(Duration::from_millis(1000));
    let chain_id = provider.get_chainid().await?.as_u64();

    let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    println!("Wallet address: 0x{:x}", wallet.address());
    
    let eth_client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

    let contract = SwitchAssets::new(switch_address, eth_client.clone());

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
        .route("/contract/register", post(register_asset))
        .route("/contract/get_asset", post(get_asset))
        .route("/contract/get_all_assets", get(get_all_contract_assets))
        .route("/contract/get_my_assets", get(get_my_assets))
        .route("/contract/transfer", post(transfer_asset))
        .with_state(state);
    app
}
