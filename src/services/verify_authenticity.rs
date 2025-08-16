use crate::models::certificate_model::{
    Certificate, SignedCertificate,
};
use crate::config::app_state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use ethers::types::transaction::eip712::Eip712;
use ethers::{
    contract::EthEvent,
    prelude::*,
    signers::Signer,
    types::Signature,
};
use std::error::Error;
use validator::Validate;
use crate::config::app_router::{authenticity, Authenticity};


#[utoipa::path(
    post,
    path = "/verify_authenticity",
    request_body = SignedCertificate,
    responses(
        (status = 200, description = "Signature verification result", body = String),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn verify_authenticity(
    State(state): State<AppState>,
    Json(cert): Json<SignedCertificate>,
) -> Result<Json<(String, String)>, StatusCode> {
    let certificate: Certificate = cert
        .clone()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // to validate input
    cert.validate()
        .expect(&*StatusCode::BAD_REQUEST.to_string());

    // Parse the signature from hex string
    let signature_bytes = hex::decode(cert.signature.trim_start_matches("0x")).map_err(|e| {
        eprintln!("Invalid signature format: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    eprintln!("Signature Byte: {:?}", signature_bytes);

    let signature = Signature::try_from(signature_bytes.as_slice()).map_err(|e| {
        eprintln!("Signature parsing error: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    eprintln!("Signature: {:?}", signature);

    // Compute the EIP-712 digest
    let digest = certificate.encode_eip712().map_err(|e| {
        eprintln!("EIP-712 encoding error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    //this caused big issue until I removed it
    // let digest = hash_message(digest); // Prefix with \x19Ethereum Signed Message

    eprintln!("Digest: {:?}", digest);

    // Recover the signer
    let signer = signature.recover(digest).map_err(|e| {
        eprintln!("Signer recovery error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    eprintln!("Signer: {:?}", signer);
    // very important: double check to make sure the certificate owner is the signer of the signature
    assert_eq!(signer, certificate.owner);
    // Fetch the contract's owner
    let contract = Authenticity::new(state.authenticity_contract, state.eth_client.clone());

    let manufacturer: authenticity::Manufacturer = contract
        .get_manufacturer(signer)
        .call()
        .await
        .map_err(|e| {
            eprintln!("Contract call error: {:?}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    eprintln!("Manufacturer Address: {:?}", manufacturer.manufacturer_address);
    // Verify the signer matches the owner
    if signer == manufacturer.manufacturer_address {
        Ok(Json((manufacturer.manufacturer_address.to_string(), manufacturer.name)))
        //     format!(
        //     "Signature is valid! Signed by owner: {:?}",
        //     signer
        // ))
        // )
    } else {
        Ok(Json( (signer.to_string(), manufacturer.name) ))
        //     format!(
        //     "Signature is invalid. Recovered signer: {:?}, expected owner: {:?}",
        //     signer, manufacturer.manufacturer_address
        // ))
        // )
    }
}