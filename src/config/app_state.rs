use anyhow::Error;
use ethabi::ethereum_types::Address;
use ethers::middleware::{Middleware, SignerMiddleware};
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::signers::Signer;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub eth_client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    pub authenticity_contract: Address
}

impl AppState {
    pub async fn init_app_state() -> anyhow::Result<AppState, Error> {
        // Initialize Ethereum client
        let rpc_url = env::var("BASE_URL")?;
        let private_key = env::var("PRIVATE_KEY")?;

        let originality_factory: Address = env::var("CONTRACT_ADDRESS")?
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid contract address"))?;

        let provider = Provider::<Http>::try_from(&rpc_url)?.interval(Duration::from_millis(1000));
        let chain_id = provider.get_chainid().await?.as_u64();

        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let eth_client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

        // Initialize app state
        let state = AppState {
            eth_client, //wallet address could be gotten from eth_client.signer.address()
            authenticity_contract: originality_factory,
        };

        Ok(state)
    }
}

// pub async fn init_app_state() -> anyhow::Result<AppState, Error> {
//     // Initialize Ethereum client
//     let rpc_url = env::var("BASE_URL")?;
//     let private_key = env::var("PRIVATE_KEY")?;
// 
//     let originality_factory: Address = env::var("CONTRACT_ADDRESS")?
//         .parse()
//         .map_err(|_| anyhow::anyhow!("Invalid contract address"))?;
// 
//     let provider = Provider::<Http>::try_from(&rpc_url)?.interval(Duration::from_millis(1000));
//     let chain_id = provider.get_chainid().await?.as_u64();
// 
//     let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
//     let eth_client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));
// 
//     // Initialize app state
//     let state = AppState {
//         eth_client, //wallet address could be gotten from eth_client.signer.address()
//         authenticity_contract: originality_factory,
//     };
// 
//     Ok(state)
// }
