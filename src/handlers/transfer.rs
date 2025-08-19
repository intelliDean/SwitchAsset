use std::sync::Arc;
use axum::{extract::{Path, State}, Json};
use axum::http::StatusCode;
use chrono::NaiveDateTime;
use crate::{app_state::AppState, models::{Transfer, Asset, ApiResponse}, schema::{transfers, assets}};
use diesel::prelude::*;
use utoipa::path;
use crate::models::TransferByDate;

#[utoipa::path(
    get,
    path = "/transfers/{asset_id}",
    params(("asset_id" = String, Path, description = "Asset ID")),
)]
pub async fn get_transfers_by_asset(
    Path(asset_id): Path<String>,
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<Asset>>>, StatusCode> {
    let conn = &mut state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let results = assets::table
        .filter(assets::owner.eq(address))
        .load::<Asset>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}

#[utoipa::path(
    get,
    path = "/transfers_by_date",
    responses(
        (status = 200, description = "Transfer counts by date retrieved successfully", body = Vec<TransferByDate>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn get_transfers_by_date(
    state: State<Arc<AppState>>,
) -> Result<Json<Vec<TransferByDate>>, StatusCode> {
    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("DB connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let results = transfers::table
        .select((
            diesel::dsl::sql::<diesel::sql_types::Timestamp>("date_trunc('day', to_timestamp(timestamp)) as date"),
            diesel::dsl::sql::<diesel::sql_types::BigInt>("count(*) as count"),
        ))
        .group_by(diesel::dsl::sql::<diesel::sql_types::Timestamp>("date_trunc('day', to_timestamp(timestamp))"))
        .order(diesel::dsl::sql::<diesel::sql_types::Timestamp>("date_trunc('day', to_timestamp(timestamp))"))
        .load::<(NaiveDateTime, i64)>(conn)
        .map_err(|e| {
            eprintln!("Transfers by date query error: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let transfer_data = results
        .into_iter()
        .map(|(date, count)| TransferByDate {
            date: date.and_utc().timestamp(),
            count,
        })
        .collect();

    Ok(axum::Json(transfer_data))
}
