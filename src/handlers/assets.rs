use crate::{
    app_state::AppState,
    models::{ApiResponse, Asset},
    schema::assets,
};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use diesel::prelude::*;
use std::sync::Arc;
use utoipa::path;

#[utoipa::path(
    get,
    path = "/assets",
    responses(
        (status = 200, description = "List all registered assets", body = ApiResponse<Vec<Asset>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn get_all_assets(
    State(state): State<Arc<AppState>>,
) -> eyre::Result<Json<ApiResponse<Vec<Asset>>>, StatusCode> {
    use self::assets::dsl::*;
    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("DB connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let results = assets.load::<Asset>(conn).map_err(|e| {
        eprintln!("Assets query error: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(ApiResponse { data: results }))
}
