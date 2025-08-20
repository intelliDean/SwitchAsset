use crate::app_route::AssetRegisteredFilter;
use crate::app_route::OwnershipTransferredFilter;
use crate::app_state::AppState;
use crate::models::{OwnershipTransferredResponse, TransferAssetInput};
use crate::schema::{assets, transfers};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use ethabi::RawLog;
use ethabi::ethereum_types::{H160, H256};
use ethers::contract::EthEvent;
use ethers::prelude::*;
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/contract/transfer",
    request_body(content = TransferAssetInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Asset transferred successfully", body = OwnershipTransferredResponse),
        (status = 400, description = "Invalid asset ID, new owner address, or ownership issue"),
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
    let wallet_address = contract.client().address();
    let balance = contract
        .client()
        .get_balance(wallet_address, None)
        .await
        .map_err(|e| {
            eprintln!("Balance check error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    eprintln!(
        "Wallet address: 0x{:x}, Balance: {} wei (~{} ETH)",
        wallet_address,
        balance,
        balance.as_u128() as f64 / 1e18
    );

    let asset = contract
        .get_asset(asset_id.into())
        .call()
        .await
        .map_err(|e| {
            eprintln!("Failed to get asset {}: {:?}", hex::encode(asset_id), e);
            StatusCode::BAD_REQUEST
        })?;
    if asset.asset_owner != wallet_address {
        eprintln!(
            "Wallet 0x{:x} is not the owner of asset {}",
            wallet_address,
            hex::encode(asset_id)
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let gas_estimate = contract
        .transfer_asset(<[u8; 32]>::from(asset_id), new_owner)
        .estimate_gas()
        .await
        .map_err(|e| {
            eprintln!(
                "Gas estimation error for asset_id {}: {:?}",
                hex::encode(asset_id),
                e
            );
            StatusCode::BAD_REQUEST
        })?;
    let gas_limit = gas_estimate * 120 / 100;

    eprintln!(
        "Estimated gas: {}, Set gas limit: {}",
        gas_estimate, gas_limit
    );

    let gas_price = contract
        .client()
        .get_gas_price()
        .await
        .unwrap_or(U256::from(2_000_000_000u64));

    eprintln!(
        "Gas price: {} wei ({} Gwei)",
        gas_price,
        gas_price.as_u64() as f64 / 1e9
    );

    let required_funds: U256 = gas_limit * gas_price;
    eprintln!(
        "Required funds: {} wei (~{} ETH)",
        required_funds,
        required_funds.as_u128() as f64 / 1e18
    );

    if balance < required_funds {
        eprintln!(
            "Insufficient funds: have {} wei, need {} wei",
            balance, required_funds
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    let call = contract
        .transfer_asset(<[u8; 32]>::from(asset_id), new_owner)
        .gas(gas_limit)
        .gas_price(gas_price)
        .value(U256::zero());

    let tx = call
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
            let transaction_hash = format!("0x{}", hex::encode(tx.transaction_hash));
            let timestamp = chrono::Utc::now().timestamp();

            let conn = &mut state.db_pool.get().map_err(|e| {
                eprintln!("DB connection error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            // Check if transfer exists
            let exists: bool = transfers::table
                .filter(transfers::asset_id.eq(&db_asset_id))
                .filter(transfers::txn_hash.eq(&transaction_hash))
                .select(diesel::dsl::count_star())
                .first::<i64>(conn)
                .unwrap()
                > 0;

            if exists {
                eprintln!(
                    "Skipping duplicate transfer for asset {} (tx: {})",
                    db_asset_id, transaction_hash
                );
                continue;
            }

            diesel::insert_into(transfers::table)
                .values((
                    transfers::asset_id.eq(&db_asset_id),
                    transfers::old_owner.eq(&db_old_owner),
                    transfers::new_owner.eq(&db_new_owner),
                    transfers::timestamp.eq(timestamp),
                    transfers::txn_hash.eq(&transaction_hash),
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
            println!("    Tx Hash: {}", transaction_hash);
        }
    }

    Ok(Json(event_res))
}
