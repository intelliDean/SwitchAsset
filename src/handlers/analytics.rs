use crate::models::{Analytics, TopOwner};
use crate::schema::{assets, transfers};
use crate::app_state::AppState;
use axum::Json;
use diesel::prelude::*;
use diesel::prelude::*;
use eyre::Result;
use serde_json::json;
use std::fs::File;
use std::io::{BufReader, Write};
use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/analytics",
    responses(
        (status = 200, description = "Analytics data retrieved successfully", body = Analytics),
        (status = 500, description = "Internal server error")
    ),
    tag = "SwitchAssets"
)]
pub async fn get_analytics() -> Result<Json<Analytics>, axum::http::StatusCode> {
    let file = File::open("src/files/analytics.json").map_err(|e| {
        eprintln!("Failed to open analytics.json: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let reader = BufReader::new(file);
    let analytics: Analytics = serde_json::from_reader(reader).map_err(|e| {
        eprintln!("Failed to parse analytics.json: {:?}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(analytics))
}


pub async fn generate_analytics(state: &AppState) -> Result<()> {
    let conn = &mut state.db_pool.get()?;

    // Total assets registered
    let total_assets: i64 = assets::table.count().get_result(conn)?;

    // Total ownership transfers
    let total_transfers: i64 = transfers::table.count().get_result(conn)?;

    // Top 3 most active owners by number of transfers
    let top_owners = transfers::table
        .select((
            transfers::new_owner,
            diesel::dsl::sql::<diesel::sql_types::BigInt>("count(*) as transfer_count"),
        ))
        .group_by(transfers::new_owner)
        .order(diesel::dsl::sql::<diesel::sql_types::BigInt>("count(*) DESC"))
        .limit(3)
        .load::<(String, i64)>(conn)?
        .into_iter()
        .map(|(owner, transfer_count)| TopOwner { owner, transfer_count })
        .collect::<Vec<_>>();

    // Create analytics struct
    let analytics = Analytics {
        total_assets,
        total_transfers,
        top_owners,
    };

    // Export to JSON
    let json_data = json!({
        "total_assets": analytics.total_assets,
        "total_transfers": analytics.total_transfers,
        "top_owners": analytics.top_owners
    });
    let mut json_file = File::create("src/files/analytics.json")?;
    serde_json::to_writer_pretty(&mut json_file, &json_data)?;
    json_file.flush()?;

    // Export to Markdown
    let mut md_file = File::create("src/files/summary.md")?;
    writeln!(md_file, "# SwitchAssets Analytics Summary")?;
    writeln!(md_file, "## Total Assets Registered")?;
    writeln!(md_file, "{}", analytics.total_assets)?;
    writeln!(md_file, "## Total Ownership Transfers")?;
    writeln!(md_file, "{}", analytics.total_transfers)?;
    writeln!(md_file, "## Top 3 Most Active Owners")?;
    for owner in &analytics.top_owners {
        writeln!(md_file, "- {}: {} transfers", owner.owner, owner.transfer_count)?;
    }
    md_file.flush()?;

    Ok(())
}


