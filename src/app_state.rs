use crate::app_route::SwitchAssets;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use ecdsa::SigningKey;
use ethers::core::k256::Secp256k1;
use ethers::{prelude::*, providers::Http};


#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub contract: SwitchAssets<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>
}