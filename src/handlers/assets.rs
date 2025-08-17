use axum::{extract::State, Json};
use axum::http::StatusCode;
use crate::{app_state::AppState, models::{Asset, ApiResponse}, schema::assets};
use diesel::prelude::*;
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
    State(state): State<AppState>,
) -> eyre::Result<Json<ApiResponse<Vec<Asset>>>, StatusCode> {
    use self::assets::dsl::*;
    let conn = &mut state
        .db_pool
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let results = assets
        .load::<Asset>(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ApiResponse { data: results }))
}
