use crate::app_route::SwitchAssets;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use ecdsa::SigningKey;
use ethers::core::k256::Secp256k1;
use ethers::{prelude::*, providers::Http};
use eyre::Report;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub contract: SwitchAssets<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
    // pub last_processed_block: ()
}

impl AppState {
    pub async fn init() -> eyre::Result<AppState, Report> {
        dotenv().ok();

        //db connection
        let db_url = env::var("DATABASE_URL")?;
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| eyre::eyre!("Failed to create pool: {}", e))?;

        //contract connection
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
}
