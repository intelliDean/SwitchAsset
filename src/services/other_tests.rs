use crate::models::certificate_model::RegInput;
use crate::models::certificate_model::{Certificate, CertificateData};
use crate::models::events::ManufacturerRegistered;
use crate::config::app_router::{Authenticity, authenticity};
use crate::config::app_state::AppState;
use axum::{Json, extract::Path, extract::State, http::StatusCode};
use ethabi::RawLog;
use ethers::types::transaction::eip712::Eip712;
use ethers::{contract::EthEvent, prelude::*, signers::Signer, types::Signature};
use std::error::Error;

//============== FOR TEST ONLY => WILL BE REMOVED WHEN DONE =======================

#[utoipa::path(
    post,
    path = "/manufacturer_registers", //TODO: Registration will be done from the frontend
    request_body = RegInput,
    responses(
        (status = 200, description = "Signature verification result", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn manufacturer_registers(
    State(state): State<AppState>,
    Json(input): Json<RegInput>,
) -> Result<Json<String>, StatusCode> {
    // Fetch the contract's owner
    let contract = Authenticity::new(state.authenticity_contract, state.eth_client.clone());

    let receipt = contract
        .manufacturer_registers(input.name)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Transaction send error: {:?}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .await
        .map_err(|e| {
            eprintln!("Transaction confirmation error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if receipt.status != Some(1.into()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut event_res = ManufacturerRegistered::init();

    for log in receipt.logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.clone().to_vec(),
        };

        if let Ok(event) = <ManufacturerRegistered as EthEvent>::decode_log(&raw_log) {
            event_res = ManufacturerRegistered::new(
                event.manufacturer_address,
                event.manufacturer_name.clone(),
            );

            println!("📦 Manufacturer Registers:");
            println!("    Manufacturer Address: {}", event.manufacturer_address);
            println!("    Manufacturer Name: {:?}", event.manufacturer_name);
        }
    }

    Ok(Json(format!(
        "Manufacturer Address: {:?}, Manufacturer Name: {:?}",
        event_res.manufacturer_address, event_res.manufacturer_name
    )))
}

#[utoipa::path( //TODO: This was just used to check the contract status
    get,
    path = "/get_owner/{address}",
    params(
        ("address" = String, Path, description = "Address of the owner")
    ),
    responses(
        (status = 200, description = "Owner retrieved successfully", body = String),
        (status = 400, description = "Invalid Owner Address"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_owner(
    State(state): State<AppState>,
    Path(input): Path<String>,
) -> Result<Json<Address>, StatusCode> {
    let contract = Authenticity::new(state.authenticity_contract, state.eth_client.clone());

    let owner = input.parse().unwrap();
    let manufacturer_address = contract
        .get_manufacturer_address(owner)
        .call()
        .await
        .map_err(|e| {
            eprintln!("Contract call error: {:?}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(manufacturer_address))
}

#[utoipa::path( //TODO: This will be called from the frontend, just created this for test
    post,
    path = "/verify_signature",
    request_body = CertificateData,
    responses(
        (status = 200, description = "Signature verified on-chain successfully", body = String),
        (status = 400, description = "Invalid signature"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn verify_signature(
    State(state): State<AppState>,
    Json(cert): Json<CertificateData>,
) -> anyhow::Result<Json<String>, StatusCode> {

    let certificate: Certificate = cert
        .clone()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // accessing the wallet from SignerMiddleware
    // Sign the certificate
    let signature: Signature = state
        .eth_client
        .signer()
        .sign_typed_data(&certificate)
        .await
        .map_err(|e| {
            eprintln!("Signature error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    eprintln!("Signature: {:?}", signature);

    // Convert to contract certificate
    let contract_cert: authenticity::Certificate = certificate.into();
    // let sig_bytes = to_bytes(signature);

    // Call create_item
    let contract = Authenticity::new(state.authenticity_contract, state.eth_client.clone());

    eprintln!("Address: {:?}", state.eth_client.signer().address());
    let bytes_sign = Bytes::from(signature.to_vec());

    eprintln!("Bytes Signature: {:?}", bytes_sign);
    eprintln!("Certificate: {:?}", contract_cert);

    let result = contract
        .verify_signature(contract_cert, bytes_sign)
        .call()
        .await
        .map_err(|e| {
            eprintln!("Transaction send error: {:?}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    eprintln!("Result: {:?}", result);

    Ok(Json(format!("Result: {:?}", result)))
}

#[utoipa::path( //TODO: This is purely for testing purpose
    post,
    path = "/generate_signature",
    request_body = CertificateData,
    responses(
        (status = 200, description = "Signature verification result", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn generate_signature(
    State(state): State<AppState>,
    Json(cert): Json<CertificateData>,
) -> Result<Json<String>, StatusCode> {
    let certificate: Certificate = cert
        .clone()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let signature: Signature = state
        .eth_client
        .signer()
        .sign_typed_data(&certificate)
        .await
        .map_err(|e| {
            eprintln!("Signature error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json("0x".to_owned() + &*signature.to_string()))
}
