use crate::schema::{assets, transfers};
use axum::{Json, extract::State, http::StatusCode};
use diesel::prelude::*;
use ethers::{prelude::*, types::Log};
use eyre::Result;
// use crate::state::AppState;
use crate::app_route::{AssetRegisteredFilter, OwnershipTransferredFilter};
use crate::app_state::AppState;
use crate::models::{
    ApiResponse, Asset as DbAsset, AssetRegisteredResponse, GetAssetInput,
    OwnershipTransferredResponse, RegisterAssetInput, TransferAssetInput,
};
use crate::schema::assets::description;
use chrono::Utc;
use ethabi::RawLog;
use serde::{Deserialize, Serialize};
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
    State(state): State<AppState>,
    Json(input): Json<RegisterAssetInput>,
) -> Result<Json<String>, StatusCode> {
    // let contract = state.contract.clone();
    //
    // // Estimate gas for the transaction
    // let gas_estimate = contract
    //     .register_asset(input.description.clone())
    //     .estimate_gas()
    //     .await
    //     .map_err(|e| {
    //         eprintln!("Gas estimation error: {:?}", e.to_string());
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;
    //
    // // Set a gas limit with a buffer (e.g., 20% more than estimated)
    // let gas_limit = gas_estimate * 120 / 100;
    // eprintln!(
    //     "Estimated gas: {}, Set gas limit: {}",
    //     gas_estimate, gas_limit
    // );
    //
    // eprintln!("Input: {:?}", input.description.clone());
    //
    // let call = contract
    //     .register_asset(input.description.clone())
    //     .gas(gas_limit)
    //     .gas_price(gas_price)
    //     .value(U256::zero());
    // let tx = call
    //     .send()
    //     .await
    //     .map_err(|e| {
    //         eprintln!("Transaction send error: {:?}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?
    //     .await
    //     .map_err(|e| {
    //         eprintln!("Transaction confirmation error: {:?}", e);
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?
    //     .ok_or_else(|| {
    //         eprintln!("No transaction receipt");
    //         StatusCode::INTERNAL_SERVER_ERROR
    //     })?;
    //
    // eprintln!("Transaction: {:?}", tx);
    //
    // if tx.status != Some(1.into()) {
    //     eprintln!("Transaction failed: {:?}", tx);
    //     return Err(StatusCode::BAD_REQUEST);
    // }
    //
    // if tx.status != Some(1.into()) {
    //     eprintln!("Transaction failed: {:?}", tx);
    //     return Err(StatusCode::BAD_REQUEST);
    // }

    let contract = state.contract.clone();

    // Log input for debugging
    eprintln!("Registering asset with description: {}", input.description);

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

    // Ok(Json("Done".to_string()))
}

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
    State(state): State<AppState>,
    Json(input): Json<GetAssetInput>,
) -> Result<Json<ApiResponse<DbAsset>>, StatusCode> {
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
        data: DbAsset {
            asset_id: format!("0x{}", hex::encode(asset.asset_id)),
            owner: format!("0x{}", hex::encode(asset.asset_owner)),
            description: asset.description.to_string(),
            registered_at: asset.registered_at.as_u64() as i64,
        },
    }))
}

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
    State(state): State<AppState>,
) -> Result<Json<Vec<DbAsset>>, StatusCode> {
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
        let db_asset = DbAsset {
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

#[utoipa::path(
    get,
    path = "/contract/get_my_assets",
    responses(
        (status = 200, description = "User's assets retrieved successfully", body = ApiResponse<Vec<DbAsset>>),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]

pub async fn get_my_assets(
    State(state): State<AppState>,
) -> Result<Json<Vec<DbAsset>>, StatusCode> {
    let contract = state.contract.clone();
    let assets_tuple = contract.get_my_assets().call().await.map_err(|e| {
        eprintln!("getMyAssets call error: {:?}", e.to_string());
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let db_assets = assets_tuple
        .into_iter()
        .map(|asset| DbAsset {
            asset_id: format!("0x{}", hex::encode(asset.asset_id)),
            owner: format!("0x{}", hex::encode(asset.asset_owner)),
            description: asset.description.to_string(),
            registered_at: asset.registered_at.as_u64() as i64,
        })
        .collect();

    Ok(Json(db_assets))
}

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
// Call transferAsset and save OwnershipTransferred event to DB
pub async fn transfer_asset(
    State(state): State<AppState>,
    Json(input): Json<TransferAssetInput>,
) -> Result<Json<OwnershipTransferredResponse>, StatusCode> {
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
            // eprintln!("Transaction send error: {:?}", e);
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
