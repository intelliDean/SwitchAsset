use std::sync::Arc;
use crate::app_state::AppState;
use crate::models::{ApiResponse, Asset as DbAsset, GetAssetInput};
use axum::{Json, extract::State, http::StatusCode};
use diesel::prelude::*;
use ethabi::ethereum_types::H256;
use ethers::prelude::*;
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/contract/get_asset",
    request_body(content = GetAssetInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Asset retrieved successfully", body = ApiResponse<DbAsset>),
        (status = 400, description = "Invalid asset ID format or asset does not exist"),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Json(input): Json<GetAssetInput>,
) -> eyre::Result<Json<ApiResponse<crate::models::Asset>>, StatusCode> {
    let asset_id_bytes = hex::decode(input.asset_id.strip_prefix("0x").unwrap_or(&input.asset_id))
        .map_err(|e| {
            eprintln!("Invalid asset_id format: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
    let asset_id = H256::from_slice(&asset_id_bytes);

    let contract = state.contract.clone();
    let asset = contract
        .get_asset(<[u8; 32]>::from(asset_id))
        .call()
        .await
        .map_err(|e| {
            if e.to_string().contains("ASSET_DOES_NOT_EXIST") {
                eprintln!("Asset does not exist: {:?}", asset_id);
                StatusCode::BAD_REQUEST
            } else {
                eprintln!("getAsset call error: {:?}", e.to_string());
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json(ApiResponse {
        data: crate::models::Asset {
            asset_id: format!("0x{}", hex::encode(asset.asset_id)),
            owner: format!("0x{}", hex::encode(asset.asset_owner)),
            description: asset.description.to_string(),
            registered_at: asset.registered_at.as_u64() as i64,
        },
    }))
}
