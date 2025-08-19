use crate::app_state::AppState;
use crate::models::{ApiResponse, Asset, SearchInput};
use crate::schema::assets;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/search",
    request_body(content = SearchInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Search results retrieved successfully", body = ApiResponse<Vec<Asset>>),
        (status = 400, description = "Invalid search parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn search_events(
    state: axum::extract::State<Arc<AppState>>,
    axum::Json(input): axum::Json<SearchInput>,
) -> Result<axum::Json<ApiResponse<Vec<Asset>>>, axum::http::StatusCode> {
    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("DB connection error: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut query = assets::table.into_boxed();

    if let Some(asset_id) = input.asset_id {
        query = query.filter(assets::asset_id.eq(asset_id));
    }

    if let Some(owner_address) = input.owner_address {
        query = query.filter(assets::owner.eq(owner_address));
    }

    if let Some(start_date) = input.start_date {
        query = query.filter(assets::registered_at.ge(start_date));
    }

    if let Some(end_date) = input.end_date {
        query = query.filter(assets::registered_at.le(end_date));
    }

    let results = query
        .load::<Asset>(conn)
        .map_err(|e| {
            eprintln!("Search query error: {:?}", e);
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::Json(ApiResponse { data: results }))
}

//
// {
// "asset_id": "string",
// "end_date": 9007199254740991,
// "owner_address": "string",
// "start_date": 1755547554
// }