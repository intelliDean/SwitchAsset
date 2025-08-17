use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use ethers::{providers::Http, prelude::*, types::Address};
use crate::app_route::SwitchAssets;



#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub contract: SwitchAssets<Provider<Http>>,
}