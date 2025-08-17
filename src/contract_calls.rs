use crate::app_route::AssetRegisteredFilter;
use crate::app_state::AppState;
use crate::models::{Asset as DbAsset, Asset};
use crate::schema::{assets, transfers};
use chrono::Utc;
use diesel::prelude::*;
use ethers::{prelude::*, providers::Middleware, signers::LocalWallet};
use eyre::{eyre, Result};
use hex::encode;

// // Updated abigen with all functions
// abigen!(
//     SwitchAssets,
//     r#"[
//         function registerAsset(string memory description) public
//         function getAsset(bytes32 id) public view returns ((bytes32, address, string, uint256))
//         function getAllAssets() public view returns ((bytes32, address, string, uint256)[])
//         function getMyAssets() public view returns ((bytes32, address, string, uint256)[])
//         function transferAsset(bytes32 assetId, address newOwner) public
//         event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner)
//         event OwnershipTransferred(bytes32 indexed assetId, address indexed oldOwner, address indexed newOwner)
//     ]"#
// );

// Helper to parse Asset from tuple
// fn tuple_to_asset(asset_tuple: (H256, H160, String, U256)) -> Asset {
//     Asset {
//         asset_id: asset_tuple.0,
//         owner: asset_tuple.1,
//         description: asset_tuple.2,
//         registered_at: asset_tuple.3
//     }
// }

// Function to call registerAsset and save event to DB
pub async fn register_asset(
    state: &AppState,
    description: String,
    wallet: LocalWallet
) -> Result<DbAsset> {
    let contract = &state.contract;
    let chain_id = contract.client().chain_id().await?;
    let signer = wallet.with_chain_id(chain_id);

    // 1. Send transaction
    let tx = contract
        .register_asset(description.clone())
        .from(signer.address())
        .send()
        .await?;

    // 2. Wait for transaction receipt
    let receipt = tx
        .await?
        .ok_or_else(|| eyre!("Transaction failed - no receipt"))?;

    // 3. Parse logs for AssetRegistered event
    let mut registered_asset = None;
    for log in receipt.logs {
        if let Ok(event) = contract.decode_event::<AssetRegisteredFilter>("AssetRegistered", log.clone()) {
            // 4. Fetch asset details from contract
            let (_, _, asset_description, registered_at) = contract
                .get_asset(event.asset_id)
                .call()
                .await?;

            // 5. Prepare database model
            let db_asset = DbAsset {
                asset_id: format!("0x{:x}", event.asset_id),
                owner: format!("0x{:x}", event.asset_owner),
                description: asset_description,
                registered_at: registered_at.as_u64() as i64,
            };

            // 6. Insert into database
            let conn = &mut state.db_pool.get()?;
            diesel::insert_into(assets::table)
                .values(&db_asset)
                .execute(conn)?;

            registered_asset = Some(db_asset);
            break; // Assuming one event per tx
        }
    }

    registered_asset.ok_or_else(|| eyre!("No AssetRegistered event found in receipt"))
}


// Function to call getAsset
pub async fn get_asset(state: &AppState, id: H256) -> Result<Asset> {
    let contract = &state.contract;
    let asset_tuple = contract.get_asset(id).call().await?;
    Ok(tuple_to_asset(asset_tuple))
}

// Function to call getAllAssets and save to DB
pub async fn get_all_assets(state: &AppState) -> Result<Vec<Asset>> {
    let contract = &state.contract;
    let assets_tuple = contract.get_all_assets().call().await?;
    let mut db_assets = Vec::new();

    for asset in &assets_tuple {
        let asset_id = format!("0x{}", encode(asset.0));
        let owner = format!("0x{}", encode(asset.1.0));
        db_assets.push(DbAsset {
            asset_id: asset_id.clone(),
            owner,
            description: asset.2.clone(),
            registered_at: asset.3.as_u64() as i64,
        });
    }

    let conn = &mut state.db_pool.get()?;
    for db_asset in db_assets {
        diesel::insert_or_ignore_into(assets::table)
            .values(&db_asset)
            .execute(conn)?;
    }

    Ok(assets_tuple.into_iter().map(tuple_to_asset).collect())
}

// Function to call getMyAssets
pub async fn get_my_assets(state: &AppState) -> Result<Vec<Asset>> {
    let contract = &state.contract;
    let assets_tuple = contract.get_my_assets().call().await?;
    Ok(assets_tuple.into_iter().map(tuple_to_asset).collect())
}

// Function to call transferAsset and save event to DB
pub async fn transfer_asset(state: &AppState, asset_id: H256, new_owner: H160, wallet: LocalWallet) -> Result<()> {
    let contract = &state.contract;
    let signer = wallet.with_chain_id(contract.client().chain_id().await?);
    let tx = contract.transfer_asset(asset_id, new_owner).from(signer.address()).send().await?;
    let receipt = tx.await?.ok_or(eyre::eyre!("No receipt"))?;

    // Parse logs for OwnershipTransferred event
    for log in receipt.logs {
        if let Some(event) = contract.decode_event::<OwnershipTransferredFilter>("OwnershipTransferred", log.clone())? {
            let db_asset_id = format!("0x{}", encode(event.asset_id));
            let db_old_owner = format!("0x{}", encode(event.old_owner.0));
            let db_new_owner = format!("0x{}", encode(event.new_owner.0));
            let timestamp = Utc::now().timestamp();

            let conn = &mut state.db_pool.get()?;
            diesel::insert_into(transfers::table)
                .values((
                    transfers::asset_id.eq(db_asset_id.clone()),
                    transfers::old_owner.eq(db_old_owner),
                    transfers::new_owner.eq(db_new_owner.clone()),
                    transfers::timestamp.eq(timestamp),
                ))
                .execute(conn)?;

            diesel::update(assets::table)
                .filter(assets::asset_id.eq(db_asset_id))
                .set(assets::owner.eq(db_new_owner))
                .execute(conn)?;
        }
    }

    Ok(())
}