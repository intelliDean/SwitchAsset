// use crate::app_route::SwitchAssetsEvents;
// use crate::app_state::AppState;
// use crate::schema::{assets, transfers};
// use diesel::prelude::*;
// use ethers::{core::utils::to_checksum, prelude::*};
// use eyre::Result;
//
//
// pub async fn listen_for_events(state: AppState) -> Result<()> {
//     let contract = &state.contract;
//     let events = contract.events().from_block(BlockNumber::Latest);
//
//     let mut stream = events.stream().await?;
//
//     while let Some(event) = stream.next().await {
//         match event {
//             Ok(SwitchAssetsEvents::AssetRegisteredFilter(event)) => {
//                 let asset_id = format!("0x{}", hex::encode(event.asset_id));
//                 let owner = to_checksum(&event.asset_owner, None);
//                 let asset = contract
//                     .method::<_, (H256, H160, String, U256)>("getAsset", event.asset_id)?
//                     .call()
//                     .await
//                     .map_err(|e| eyre::eyre!("Failed to call getAsset: {}", e))?;
//                 let description = asset.2;
//                 let registered_at = asset.3.as_u64() as i64;
//
//                 let conn = &mut state
//                     .db_pool
//                     .get()
//                     .map_err(|e| eyre::eyre!("Failed to get DB connection: {}", e))?;
//                 diesel::insert_into(assets::table)
//                     .values((
//                         assets::asset_id.eq(&asset_id),
//                         assets::owner.eq(&owner),
//                         assets::description.eq(description),
//                         assets::registered_at.eq(registered_at),
//                     ))
//                     .execute(conn)
//                     .map_err(|e| eyre::eyre!("Failed to insert asset: {}", e))?;
//             }
//             Ok(SwitchAssetsEvents::OwnershipTransferredFilter(event)) => {
//                 let asset_id = format!("0x{}", hex::encode(event.asset_id));
//                 let old_owner = to_checksum(&event.old_owner, None);
//                 let new_owner = to_checksum(&event.new_owner, None);
//                 let timestamp = chrono::Utc::now().timestamp();
//
//                 let conn = &mut state
//                     .db_pool
//                     .get()
//                     .map_err(|e| eyre::eyre!("Failed to get DB connection: {}", e))?;
//
//                 // diesel::insert_into(assets::table)
//                 //     .values(&db_asset)
//                 //     .on_conflict(assets::asset_id)
//                 //     .do_update()
//                 //     .set(&db_asset)
//                 //     .execute(conn)
//                 //     .map_err(|e| {
//                 //         eprintln!("Failed to insert/update asset {}: {:?}", asset_id, e);
//                 //         eyre::eyre!("Failed to insert/update asset: {}", e)
//                 //     })?;
//
//
//                 diesel::insert_into(transfers::table)
//                     .values((
//                         transfers::asset_id.eq(&asset_id),
//                         transfers::old_owner.eq(&old_owner),
//                         transfers::new_owner.eq(&new_owner),
//                         transfers::timestamp.eq(timestamp),
//                     ))
//                     .execute(conn)
//                     .map_err(|e| eyre::eyre!("Failed to insert transfer: {}", e))?;
//
//                 diesel::update(assets::table)
//                     .filter(assets::asset_id.eq(&asset_id))
//                     .set(assets::owner.eq(&new_owner))
//                     .execute(conn)
//                     .map_err(|e| eyre::eyre!("Failed to update asset owner: {}", e))?;
//             }
//             Err(e) => {
//                 println!("Event stream error: {:?}", e);
//             }
//         }
//     }
//     Ok(())
// }

use crate::app_route::SwitchAssetsEvents;
use crate::app_state::AppState;
use crate::schema::{assets, transfers};
use diesel::prelude::*;
use ethers::{core::utils::to_checksum, prelude::*};
use eyre::Result;

pub async fn listen_for_events(state: AppState) -> Result<()> {
    let contract = &state.contract;
    let events = contract.events().from_block(BlockNumber::Latest);

    let mut stream = events.stream().await.map_err(|e| {
        eprintln!("Failed to create event stream: {:?}", e);
        eyre::eyre!("Failed to create event stream: {}", e)
    })?;

    loop {
        match stream.next().await {
            Some(Ok(SwitchAssetsEvents::AssetRegisteredFilter(event))) => {
                let asset_id = format!("0x{}", hex::encode(event.asset_id));
                let owner = to_checksum(&event.asset_owner, None);

                // Call getAsset to fetch additional details
                let asset = contract
                    .method::<_, (H256, H160, String, U256)>("getAsset", event.asset_id)?
                    .call()
                    .await
                    .map_err(|e| {
                        if e.to_string().contains("ASSET_DOES_NOT_EXIST") {
                            eprintln!("Asset does not exist for ID: {}", asset_id);
                            eyre::eyre!("Asset does not exist for ID: {}", asset_id)
                        } else {
                            eprintln!("Failed to call getAsset for ID {}: {:?}", asset_id, e);
                            eyre::eyre!("Failed to call getAsset: {}", e)
                        }
                    })?;

                let description = asset.2;
                let registered_at = asset.3.as_u64() as i64;

                // Use a single DB connection for the operation
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;

                // Insert or update asset
                let db_asset = (
                    assets::asset_id.eq(&asset_id),
                    assets::owner.eq(&owner),
                    assets::description.eq(&description),
                    assets::registered_at.eq(registered_at),
                );
                diesel::insert_into(assets::table)
                    .values(&db_asset)
                    .on_conflict(assets::asset_id)
                    .do_update()
                    .set(db_asset)
                    .execute(conn)
                    .map_err(|e| {
                        eprintln!("Failed to insert/update asset {}: {:?}", asset_id, e);
                        eyre::eyre!("Failed to insert/update asset: {}", e)
                    })?;

                eprintln!("📦 Asset Registered: ID = {}, Owner = {}", asset_id, owner);
            }

            Some(Ok(SwitchAssetsEvents::OwnershipTransferredFilter(event))) => {
                let asset_id = format!("0x{}", hex::encode(event.asset_id));
                let old_owner = to_checksum(&event.old_owner, None);
                let new_owner = to_checksum(&event.new_owner, None);
                let timestamp = chrono::Utc::now().timestamp();

                // Use a single DB connection and transaction
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;

                conn.transaction(|conn| {
                    // Insert transfer record
                    diesel::insert_into(transfers::table)
                        .values((
                            transfers::asset_id.eq(&asset_id),
                            transfers::old_owner.eq(&old_owner),
                            transfers::new_owner.eq(&new_owner),
                            transfers::timestamp.eq(timestamp),
                        ))
                        .execute(conn)
                        .map_err(|e| {
                            eprintln!("Failed to insert transfer for asset {}: {:?}", asset_id, e);
                            eyre::eyre!("Failed to insert transfer: {}", e)
                        })?;

                    // Update asset owner
                    diesel::update(assets::table)
                        .filter(assets::asset_id.eq(&asset_id))
                        .set(assets::owner.eq(&new_owner))
                        .execute(conn)
                        .map_err(|e| {
                            eprintln!("Failed to update asset owner for {}: {:?}", asset_id, e);
                            eyre::eyre!("Failed to update asset owner: {}", e)
                        })?;

                    Ok::<(), eyre::Report>(())
                })
                    .map_err(|e| {
                        eprintln!("Transaction failed for asset {}: {:?}", asset_id, e);
                        eyre::eyre!("Transaction failed: {}", e)
                    })?;

                eprintln!(
                    "📦 Ownership Transferred: ID = {}, Old Owner = {}, New Owner = {}",
                    asset_id, old_owner, new_owner
                );
            }
            Some(Err(e)) => {
                eprintln!("Event stream error: {:?}", e);
                // Optional: Add retry logic or delay
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            None => {
                eprintln!("Event stream ended unexpectedly");
                break;
            }
        }
    }

    Ok(())
}