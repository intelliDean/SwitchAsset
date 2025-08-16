use axum::Router;

use anyhow::Result;
use dotenv::dotenv;
use crate::models::router_path::RouterPath;
use crate::config::app_router::paths;
use crate::config::app_state::{AppState};

pub async fn server() -> Result<()> {
    eprintln!("PROJECT STARTING...");
    // Load environment variables
    dotenv().ok();
    // dotenv::from_path("../.env").ok();

    let state = AppState::init_app_state().await?; //init_app_state().await?;

    // Define routes
    let app: Router = paths(state, RouterPath::init());

    eprintln!("Project started and listening on 127.0.0.1:8080");

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;

    Ok(()) // another way to say return nothing
}
