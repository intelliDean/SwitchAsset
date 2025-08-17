use axum::{extract::{Path, State}, Json};
use axum::http::StatusCode;
use crate::{app_state::AppState, models::{Transfer, Asset, ApiResponse}, schema::{transfers, assets}};
use diesel::prelude::*;
use utoipa::path;

#[utoipa::path(
    get,
    path = "/transfers/{asset_id}",
    params(("asset_id" = String, Path, description = "Asset ID")),
)]
pub async fn get_transfers_by_asset(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Transfer>>>, StatusCode> {
    let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results = transfers::table
        .filter(transfers::asset_id.eq(asset_id))
        .load::<Transfer>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}

#[utoipa::path(
    get,
    path = "/assets/owner/{address}",
    params(("address" = String, Path, description = "Owner address")),
)]
pub async fn get_assets_by_owner(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Asset>>>, StatusCode> {
    let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let results = assets::table
        .filter(assets::owner.eq(address))
        .load::<Asset>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}
