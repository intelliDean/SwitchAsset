use crate::app_state::AppState;
use crate::models::ApiResponse;
use crate::models::Asset as DbAsset;
use crate::schema::assets;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use diesel::RunQueryDsl;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/contract/get_all_assets",
    responses(
        (status = 200, description = "All assets retrieved successfully", body = ApiResponse<Vec<DbAsset>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]

pub async fn get_all_contract_assets(
    State(state): State<Arc<AppState>>,
) -> eyre::Result<Json<Vec<crate::models::Asset>>, StatusCode> {
    let contract = state.contract.clone();
    let assets_tuple = contract.get_all_assets().call().await.map_err(|e| {
        eprintln!("getAllAssets call error: {:?}", e.to_string());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut db_assets = Vec::new();
    let conn = &mut state.db_pool.get().map_err(|e| {
        eprintln!("DB connection error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    for asset in assets_tuple.iter() {
        let db_asset = crate::models::Asset {
            asset_id: format!("0x{}", hex::encode(asset.asset_id)),
            owner: format!("0x{}", hex::encode(asset.asset_owner)),
            description: asset.description.to_string(),
            registered_at: asset.registered_at.as_u64() as i64,
        };
        diesel::insert_into(assets::table)
            .values(&db_asset)
            .on_conflict(assets::asset_id)
            .do_update()
            .set(&db_asset)
            .execute(conn)
            .map_err(|e| {
                eprintln!("Failed to insert asset: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        db_assets.push(db_asset);
    }

    Ok(Json(db_assets))
}
