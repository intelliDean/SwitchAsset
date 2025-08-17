use axum::{Router, routing::get};
use crate::handlers::{assets::get_all_assets, transfers::{get_transfers_by_asset, get_assets_by_owner}};
use crate::app_state::AppState;

pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/assets", get(get_all_assets))
        .route("/transfers/:asset_id", get(get_transfers_by_asset))
        .route("/assets/owner/:address", get(get_assets_by_owner))
        .with_state(state)
}
