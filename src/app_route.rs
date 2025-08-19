use std::sync::Arc;
use crate::app_state::AppState;
use crate::contract_calls::{
    get_all_contract_assets::get_all_contract_assets, get_asset::get_asset,
    get_my_assets::get_my_assets, register_asset::register_asset, transfer_asset::transfer_asset,
};
use crate::handlers::{
    assets::get_all_assets,
    search::search_events,
    transfer::{get_assets_by_owner, get_transfers_by_asset, get_transfers_by_date},
    analytics::get_analytics
};

use crate::swagger::ApiDoc;
use axum::{
    Router,
    routing::{get, post},
};
use ethers::contract::abigen;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

//abi path
abigen!(
    SwitchAssets,
    "./hh-artifacts/contracts/SwitchAssets.sol/SwitchAssets.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub fn app_router(state: Arc<AppState>) -> Router {
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
        .route("/search", post(search_events))
        .route("/transfers_by_date", get(get_transfers_by_date))
        .route("/analytics", get(get_analytics))
        .route(
            "/chart",
            get(|| async {
                axum::response::Html(
                    std::fs::read_to_string("static/index.html")
                        .unwrap_or("<h1>Error: Chart not found</h1>".to_string()),
                )
            }),
        )
        .with_state(state);

    app
}
