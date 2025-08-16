use crate::config::swagger_config::ApiDoc;
use crate::models::router_path::RouterPath;
use crate::services::verify_authenticity::verify_authenticity;
use crate::services::create_eip712::create_certificate;
use crate::services::qr_code::generate_qr_code;
use crate::services::other_tests::{
    generate_signature, get_owner, manufacturer_registers,
    verify_signature,
};
use crate::config::app_state::AppState;
use axum::Router;
use axum::routing::{get, post};
use ethers::contract::abigen;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

//abi path
abigen!(
    Authenticity,
    "./hh-artifacts/contracts/SwitchAssets.sol.sol/SwitchAssets.sol.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub fn paths(state: AppState, path: RouterPath) -> Router {
    let app = Router::new()
        .route(&path.generate_signature, post(generate_signature))
        .route(&path.verify_authenticity, post(verify_authenticity))
        .route(&path.sign_up, post(manufacturer_registers))
        .route(&path.get_owner, get(get_owner))
        .route(&path.verify_signature, post(verify_signature))
        .route(&path.create_certificate, post(create_certificate))
        .route(&path.qr_code, post(generate_qr_code))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
        .layer(CorsLayer::permissive()); // Optional: Enable CORS

    app
}
