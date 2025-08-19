use std::sync::Arc;
// use crate::state::AppState;
use crate::app_route::{AssetRegisteredFilter};
use crate::app_state::AppState;
use crate::models::{
    Asset as DbAsset, AssetRegisteredResponse
    , RegisterAssetInput,
};
use crate::schema::assets;
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use ethabi::RawLog;
use ethers::prelude::*;
use eyre::Result;
use utoipa::ToSchema;

#[utoipa::path(
    post,
    path = "/contract/register",
    request_body(content = RegisterAssetInput, content_type = "application/json"),
    responses(
        (status = 200, description = "Asset registered successfully", body = String),
        (status = 400, description = "Transaction failed"),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn register_asset(
    State(state): State<Arc<AppState>>,
    Json(input): Json<RegisterAssetInput>,
) -> Result<Json<String>, StatusCode> {

    let contract = state.contract.clone();

    // Check wallet balance
    let wallet_address = contract.client().address();
    let balance = contract
        .client()
        .get_balance(wallet_address, None)
        .await
        .map_err(|e| {
            eprintln!("Balance check error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    eprintln!("Wallet address: 0x{:x}, Balance: {} wei (~{} ETH)", wallet_address, balance, balance.as_u128() as f64 / 1e18);

    // Estimate gas
    let gas_estimate = contract
        .register_asset(input.description.clone())
        .estimate_gas()
        .await
        .map_err(|e| {
            eprintln!("Gas estimation error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let gas_limit = gas_estimate * 120 / 100; // 120% buffer

    eprintln!("Estimated gas: {}, Set gas limit: {}", gas_estimate, gas_limit);

    // Set gas price (fallback to 2 Gwei if network fetch fails)
    let gas_price = contract
        .client()
        .get_gas_price()
        .await
        .unwrap_or(U256::from(2_000_000_000u64));

    eprintln!("Gas price: {} wei ({} Gwei)", gas_price, gas_price.as_u64() as f64 / 1e9);

    // Calculate required funds
    let required_funds: U256 = gas_limit * gas_price;
    eprintln!("Required funds: {} wei (~{} ETH)", required_funds, required_funds.as_u128() as f64 / 1e18);

    if balance < required_funds {
        eprintln!(
            "Insufficient funds: have {} wei, need {} wei",
            balance, required_funds
        );
        return Err(StatusCode::BAD_REQUEST);
    }

    // Send transaction
    let call = contract
        .register_asset(input.description.clone())
        .gas(gas_limit) //
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

    eprintln!("Transaction: {:?}", tx);

    if tx.status != Some(1.into()) {
        eprintln!("Transaction failed: {:?}", tx);
        return Err(StatusCode::BAD_REQUEST);
    }


    let mut event_res = AssetRegisteredResponse::init();

    for log in tx.logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.clone().to_vec(),
        };

        if let Ok(event) = <AssetRegisteredFilter as EthEvent>::decode_log(&raw_log) {
            event_res = AssetRegisteredResponse::new(H256::from(event.asset_id), event.asset_owner);

            let asset_id = format!("0x{}", hex::encode(event.asset_id));
            let owner = format!("0x{}", hex::encode(event.asset_owner.0));
            let asset = contract
                .get_asset(event.asset_id)
                .call()
                .await
                .map_err(|e| {
                    eprintln!("getAsset call error: {:?}", e.to_string());
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            let db_asset = DbAsset {
                asset_id: asset_id.clone(),
                owner,
                description: asset.description,
                registered_at: asset.registered_at.as_u64() as i64,
            };

            let conn = &mut state.db_pool.get().map_err(|e| {
                eprintln!("DB connection error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
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

            println!("📦 Asset Registered:");
            println!("    Asset ID: {}", asset_id);
            println!("    Owner: {}", db_asset.owner);
        }
    }

    Ok(Json(format!(
        "Asset ID: 0x{:x}, Owner: 0x{:x}",
        event_res.asset_id, event_res.asset_owner
    )))
}