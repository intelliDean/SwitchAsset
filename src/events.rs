use ethers::{prelude::*, core::utils::to_checksum};
use eyre::Result;
use axum::http::StatusCode;
use diesel::prelude::*;
use crate::app_state::AppState;
use crate::schema::{assets, transfers};
use crate::app_route::SwitchAssetsEvents;
// use crate::app_state;

// abigen!(
//     SwitchAssets,
//     r#"[
//         event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner)
//         event OwnershipTransferred(bytes32 indexed assetId, address indexed oldOwner, address indexed newOwner)
//         function getAsset(bytes32) view returns (bytes32, address, string, uint256)
//         function getMyAssets() view returns ((bytes32, address, string, uint256)[])
//     ]"#
// );

pub async fn listen_for_events(state: AppState) -> Result<()> {
    let contract = &state.contract;
    let events = contract.events().from_block(BlockNumber::Latest);

    let mut stream = events.stream().await?;
    while let Some(event) = stream.next().await {
        match event {
            Ok(SwitchAssetsEvents::AssetRegisteredFilter(event)) => {
                let asset_id = format!("0x{}", hex::encode(event.asset_id));
                let owner = to_checksum(&event.asset_owner, None);
                let asset = contract
                    .method::<_, (H256, H160, String, U256)>("getAsset", event.asset_id)?
                    .call()
                    .await
                    .map_err(|e| eyre::eyre!("Failed to call getAsset: {}", e))?;
                let description = asset.2;
                let registered_at = asset.3.as_u64() as i64;

                let conn = &mut state.db_pool.get().map_err(|e| eyre::eyre!("Failed to get DB connection: {}", e))?;
                diesel::insert_into(assets::table)
                    .values((
                        assets::asset_id.eq(&asset_id),
                        assets::owner.eq(&owner),
                        assets::description.eq(description),
                        assets::registered_at.eq(registered_at),
                    ))
                    .execute(conn)
                    .map_err(|e| eyre::eyre!("Failed to insert asset: {}", e))?;
            }
            Ok(SwitchAssetsEvents::OwnershipTransferredFilter(event)) => {
                let asset_id = format!("0x{}", hex::encode(event.asset_id));
                let old_owner = to_checksum(&event.old_owner, None);
                let new_owner = to_checksum(&event.new_owner, None);
                let timestamp = chrono::Utc::now().timestamp();

                let conn = &mut state.db_pool.get().map_err(|e| eyre::eyre!("Failed to get DB connection: {}", e))?;
                diesel::insert_into(transfers::table)
                    .values((
                        transfers::asset_id.eq(&asset_id),
                        transfers::old_owner.eq(&old_owner),
                        transfers::new_owner.eq(&new_owner),
                        transfers::timestamp.eq(timestamp),
                    ))
                    .execute(conn)
                    .map_err(|e| eyre::eyre!("Failed to insert transfer: {}", e))?;

                diesel::update(assets::table)
                    .filter(assets::asset_id.eq(&asset_id))
                    .set(assets::owner.eq(&new_owner))
                    .execute(conn)
                    .map_err(|e| eyre::eyre!("Failed to update asset owner: {}", e))?;
            }
            Err(e) => {
                println!("Event stream error: {:?}", e);
            }
        }
    }
    Ok(())
}