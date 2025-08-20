use crate::app_state::AppState;
use crate::models::{ApiResponse, Asset as DbAsset};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/contract/get_my_assets",
    responses(
        (status = 200, description = "User's assets retrieved successfully", body = ApiResponse<Vec<DbAsset>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]

pub async fn get_my_assets(
    State(state): State<Arc<AppState>>,
) -> eyre::Result<Json<Vec<crate::models::Asset>>, StatusCode> {
    let contract = state.contract.clone();
    let assets_tuple = contract.get_my_assets().call().await.map_err(|e| {
        eprintln!("getMyAssets call error: {:?}", e.to_string());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let db_assets = assets_tuple
        .into_iter()
        .map(|asset| crate::models::Asset {
            asset_id: format!("0x{}", hex::encode(asset.asset_id)),
            owner: format!("0x{}", hex::encode(asset.asset_owner)),
            description: asset.description.to_string(),
            registered_at: asset.registered_at.as_u64() as i64,
        })
        .collect();

    Ok(Json(db_assets))
}
