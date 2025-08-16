use config::server::server;

mod config;
mod models;
mod services;
mod utility;

#[tokio::main]
async fn main() {
    server().await.expect("Error!");
}
