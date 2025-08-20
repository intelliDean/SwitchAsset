mod app_route;
mod app_state;
mod contract_calls;
mod events;
mod handlers;
mod models;
mod schema;
mod swagger;

use crate::events::listen_for_events;

use crate::app_route::app_router;
use crate::app_state::AppState;
use crate::handlers::analytics::generate_analytics;
use diesel::prelude::*;
use diesel::prelude::*;
use ethers::{
    // Added import for to_checksum
    prelude::*,
};
use eyre::Result;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use utoipa::OpenApi;

#[tokio::main]
async fn main() -> Result<()> {
    let state = Arc::from(AppState::init().await?);

    // spawn event listener in background
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = listen_for_events(state_clone).await {
            eprintln!("Error in event listener: {:?}", e);
        }
    });

    // 2-factor auth... lol
    if let Err(e) = generate_analytics(&state).await {
        eprintln!("Analytics generation error: {:?}", e);
    }

    let app = app_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    println!("Server running on {:?}", addr);
    println!("Swagger UI available at http://127.0.0.1:8080/swagger-ui/index.html#/");

    axum::serve(listener, app).await?;

    Ok(())
}

// Part 2 – Backend & API Integration (DONE)
// Build a backend application (Node.js or Python) that can:
// • Connect to the blockchain network (Ethereum testnet preferred)
// • Listen for the events from your smart contract
// • Store event data in a local database (SQLite/PostgreSQL)
// • Provide a REST API endpoint to fetch:
// – All registered assets
// – All transfers for a given asset ID
// – All assets owned by a given address
