use crate::app_route::{AssetRegisteredFilter, OwnershipTransferredFilter};
use crate::app_route::{SwitchAssets, SwitchAssetsEvents};
use crate::app_state::AppState;
use crate::handlers::analytics::generate_analytics;
use crate::schema::{assets, transfers};
use chrono::Utc;
use diesel::prelude::*;
use ecdsa::SigningKey;
use ethers::core::k256::Secp256k1;
use ethers::{core::utils::to_checksum, prelude::*};
use eyre::Result;
use std::sync::Arc;

pub async fn listen_for_events(state: Arc<AppState>) -> Result<()> {
    let contract = state.contract.clone();
    let client = contract.client();

    // Fetch historical events from the last 1,000 blocks in chunks
    let latest_block = client.get_block_number().await.map_err(|e| {
        eprintln!("Failed to get latest block: {:?}", e);
        eyre::eyre!("Failed to get latest block: {}", e)
    })?;
    let from_block = latest_block.saturating_sub(U64::from(1000));
    let chunk_size = 499;

    // Process historical events in chunks
    let mut current_block = from_block;
    while current_block < latest_block {
        let to_block = (current_block + chunk_size).min(latest_block);
        eprintln!(
            "Querying historical events from block {} to {} (range: {})",
            current_block,
            to_block,
            to_block - current_block + 1
        );

        // Event filters for the chunk
        let asset_registered_filter = contract
            .event::<AssetRegisteredFilter>()
            .from_block(current_block)
            .to_block(to_block);

        let ownership_transferred_filter = contract
            .event::<OwnershipTransferredFilter>()
            .from_block(current_block)
            .to_block(to_block);

        // Fetch historical events with metadata
        let asset_registered_logs =
            asset_registered_filter
                .query_with_meta()
                .await
                .map_err(|e| {
                    eprintln!(
                        "Failed to query AssetRegistered events for blocks {} to {}: {:?}",
                        current_block, to_block, e
                    );
                    eyre::eyre!("Failed to query AssetRegistered events: {}", e)
                })?;
        let ownership_transferred_logs = ownership_transferred_filter
            .query_with_meta()
            .await
            .map_err(|e| {
                eprintln!(
                    "Failed to query OwnershipTransferred events for blocks {} to {}: {:?}",
                    current_block, to_block, e
                );
                eyre::eyre!("Failed to query OwnershipTransferred events: {}", e)
            })?;

        // Process historical events
        let conn = &mut state.db_pool.get().map_err(|e| {
            eprintln!("Failed to get DB connection: {:?}", e);
            eyre::eyre!("Failed to get DB connection: {}", e)
        })?;
        for (event, meta) in asset_registered_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_asset_registered_event(&contract, &event, conn, txn_hash).await?;
            if let Err(e) = generate_analytics(&state).await {
                eprintln!("Analytics generation error for AssetRegistered: {:?}", e);
            }
        }
        for (event, meta) in ownership_transferred_logs {
            let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
            process_ownership_transferred_event(&event, conn, txn_hash)?;
            if let Err(e) = generate_analytics(&state).await {
                eprintln!(
                    "Analytics generation error for OwnershipTransferred: {:?}",
                    e
                );
            }
        }

        current_block = to_block + 1;
    }

    // Stream future events
    eprintln!("Starting event stream from block {}", latest_block + 1);
    let events = contract.events().from_block(latest_block + 1);
    let mut stream = events.stream_with_meta().await.map_err(|e| {
        eprintln!("Failed to create event stream: {:?}", e);
        eyre::eyre!("Failed to create event stream: {}", e)
    })?;

    loop {
        match stream.next().await {
            Some(Ok((SwitchAssetsEvents::AssetRegisteredFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_asset_registered_event(&contract, &event, conn, txn_hash).await?;
                if let Err(e) = generate_analytics(&state).await {
                    eprintln!("Analytics generation error for AssetRegistered: {:?}", e);
                }
            }
            Some(Ok((SwitchAssetsEvents::OwnershipTransferredFilter(event), meta))) => {
                let txn_hash = Some(format!("0x{}", hex::encode(meta.transaction_hash)));
                let conn = &mut state.db_pool.get().map_err(|e| {
                    eprintln!("Failed to get DB connection: {:?}", e);
                    eyre::eyre!("Failed to get DB connection: {}", e)
                })?;
                process_ownership_transferred_event(&event, conn, txn_hash)?;
                if let Err(e) = generate_analytics(&state).await {
                    eprintln!(
                        "Analytics generation error for OwnershipTransferred: {:?}",
                        e
                    );
                }
            }
            Some(Err(e)) => {
                eprintln!("Event stream error: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
            None => {
                eprintln!("Event stream ended unexpectedly");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}

async fn process_asset_registered_event(
    contract: &SwitchAssets<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
    event: &AssetRegisteredFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
) -> Result<()> {
    let asset_id = format!("0x{}", hex::encode(event.asset_id));
    let owner = to_checksum(&event.asset_owner, None);

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

    // Check if asset exists with this txn_hash
    if let Some(ref txn) = txn_hash {
        let exists: bool = assets::table
            .filter(assets::asset_id.eq(&asset_id))
            // .filter(assets::txn_hash.eq(txn))
            .select(diesel::dsl::count_star())
            .first::<i64>(conn)
            .map(|count| count > 0)
            .map_err(|e| {
                eprintln!("Failed to check existing asset {}: {:?}", asset_id, e);
                eyre::eyre!("Failed to check existing asset: {}", e)
            })?;

        if exists {
            eprintln!(
                "Skipping duplicate asset registration for {} (tx: {})",
                asset_id, txn
            );
            return Ok(());
        }
    }

    // Insert or update the asset table
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

    Ok(())
}

fn process_ownership_transferred_event(
    event: &OwnershipTransferredFilter,
    conn: &mut PgConnection,
    txn_hash: Option<String>,
) -> Result<()> {
    let asset_id = format!("0x{}", hex::encode(event.asset_id));
    let old_owner = to_checksum(&event.old_owner, None);
    let new_owner = to_checksum(&event.new_owner, None);
    let timestamp = Utc::now().timestamp();

    // Check if transfer exists
    if let Some(ref txn) = txn_hash {
        let exists: bool = transfers::table
            .filter(transfers::asset_id.eq(&asset_id))
            .filter(transfers::txn_hash.eq(txn))
            .select(diesel::dsl::count_star())
            .first::<i64>(conn)
            .map(|count| count > 0)
            .map_err(|e| {
                eprintln!(
                    "Failed to check existing transfer for {}: {:?}",
                    asset_id, e
                );
                eyre::eyre!("Failed to check existing transfer: {}", e)
            })?;

        if exists {
            eprintln!(
                "Skipping duplicate transfer for asset {} (tx: {})",
                asset_id, txn
            );
            return Ok(());
        }
    }

    // Use transaction to update both transfers and assets
    conn.transaction(|conn| {
        // Save transfer record
        diesel::insert_into(transfers::table)
            .values((
                transfers::asset_id.eq(&asset_id),
                transfers::old_owner.eq(&old_owner),
                transfers::new_owner.eq(&new_owner),
                transfers::timestamp.eq(timestamp),
                transfers::txn_hash.eq(&txn_hash.unwrap()),
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

    Ok(())
}

//==

// Part 3 – Data Query, Analysis & Visualization (DONE)
// • Query all blockchain events related to your deployed contract for the last 1,000 blocks
// • Store the event data in a local database (SQLite/PostgreSQL)
// • Generate the following analytics:
// 1. Total number of assets ever registered
// 2. Total number of ownership transfers
// 3. Top 3 most active owners (by number of transfers)
// • Export this analysis to both:
// – A JSON file (analytics.json)
// – A Markdown summary (summary.md)
// • Create at least one chart (bar chart, pie chart, or line chart) showing activity trends over time using Chart.js, Matplotlib, or Plotly.
// • Bonus: Implement a search API endpoint (Node.js or Python FastAPI/Flask) to query stored event data by:
// – Asset ID
// – Owner address
// – Date range
