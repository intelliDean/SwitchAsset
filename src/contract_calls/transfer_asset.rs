use std::sync::Arc;
use crate::app_route::OwnershipTransferredFilter;
use crate::app_route::{AssetRegisteredFilter};
use crate::app_state::AppState;
use crate::models::{OwnershipTransferredResponse, TransferAssetInput};
use crate::schema::{assets, transfers};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use diesel::RunQueryDsl;
use ethabi::RawLog;
use ethabi::ethereum_types::{H160, H256};
use ethers::contract::EthEvent;
use diesel::prelude::*;
use ethers::prelude::*;

#[utoipa::path(
    post,
    path = "/contract/transfer",
    request_body(content = TransferAssetInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Asset transferred successfully", body = OwnershipTransferredResponse),
        (status = 400, description = "Invalid asset ID or new owner address"),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn transfer_asset(
    State(state): State<Arc<AppState>>,
    Json(input): Json<TransferAssetInput>,
) -> eyre::Result<Json<OwnershipTransferredResponse>, StatusCode> {
    let asset_id_bytes = hex::decode(input.asset_id.strip_prefix("0x").unwrap_or(&input.asset_id))
        .map_err(|e| {
            eprintln!("Invalid asset_id format: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
    let asset_id = H256::from_slice(&asset_id_bytes);
    let new_owner_bytes = hex::decode(
        input
            .new_owner
            .strip_prefix("0x")
            .unwrap_or(&input.new_owner),
    )
    .map_err(|e| {
        eprintln!("Invalid new_owner format: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;
    let new_owner = H160::from_slice(&new_owner_bytes);

    let contract = state.contract.clone();
    let tx = contract
        .transfer_asset(<[u8; 32]>::from(asset_id), new_owner)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Transaction send error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .await
        .map_err(|e| {
            eprintln!("Transaction confirmation error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            eprintln!("No transaction receipt");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if tx.status != Some(1.into()) {
        eprintln!("Transaction failed: {:?}", tx);
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut event_res = OwnershipTransferredResponse::init();

    for log in tx.logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.clone().to_vec(),
        };

        if let Ok(event) = <OwnershipTransferredFilter as EthEvent>::decode_log(&raw_log) {
            event_res = OwnershipTransferredResponse::new(
                H256::from(event.asset_id),
                event.old_owner,
                event.new_owner,
            );

            let db_asset_id = format!("0x{}", hex::encode(event.asset_id));
            let db_old_owner = format!("0x{}", hex::encode(event.old_owner.0));
            let db_new_owner = format!("0x{}", hex::encode(event.new_owner.0));
            let timestamp = chrono::Utc::now().timestamp();

            let conn = &mut state.db_pool.get().map_err(|e| {
                eprintln!("DB connection error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            diesel::insert_into(transfers::table)
                .values((
                    transfers::asset_id.eq(&db_asset_id),
                    transfers::old_owner.eq(&db_old_owner),
                    transfers::new_owner.eq(&db_new_owner),
                    transfers::timestamp.eq(timestamp),
                ))
                .execute(conn)
                .map_err(|e| {
                    eprintln!("Failed to insert transfer: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            diesel::update(assets::table)
                .filter(assets::asset_id.eq(&db_asset_id))
                .set(assets::owner.eq(&db_new_owner))
                .execute(conn)
                .map_err(|e| {
                    eprintln!("Failed to update asset owner: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            println!("📦 Ownership Transferred:");
            println!("    Asset ID: {}", db_asset_id);
            println!("    Old Owner: {}", db_old_owner);
            println!("    New Owner: {}", db_new_owner);
        }
    }

    Ok(Json(event_res))
}
