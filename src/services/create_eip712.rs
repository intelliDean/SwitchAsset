use crate::models::certificate_model::{
    Certificate, CertificateData, CustomEIP712Domain, Eip712Object,
};
use crate::utility::to_meta_hash;
use axum::{Json, http::StatusCode};
use ethers::types::transaction::eip712::Eip712;
use ethers::utils::hex::ToHexExt;
use ethers::utils::keccak256;
use ethers::{contract::EthEvent, prelude::*, signers::Signer};
use std::error::Error;

#[utoipa::path(
    post,
    path = "/create_certificate",
    request_body = CertificateData,
    responses(
        (status = 200, description = "EIP-712 object created successfully", body = Eip712Object),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_certificate(
    Json(cert): Json<CertificateData>,
) -> Result<Json<Eip712Object>, StatusCode> {
    // Validate inputs
    if cert.name.is_empty() || cert.unique_id.is_empty() || cert.serial.is_empty() {
        eprintln!("Empty name, unique_id, or serial");
        return Err(StatusCode::BAD_REQUEST);
    }
    if cert.owner.is_empty() {
        eprintln!("Empty manufacturer_address");
        return Err(StatusCode::BAD_REQUEST);
    }

    println!("owner: {:?}", cert.owner);

    // Convert to Certificate
    let certificate: Certificate = cert.try_into().map_err(|e| {
        eprintln!("Certificate conversion error: {:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Create EIP-712 domain
    let domain = certificate.domain().map_err(|e| {
        eprintln!("EIP-712 domain error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Convert to CustomEIP712Domain
    let custom_domain = CustomEIP712Domain::from(domain);

    // Define EIP-712 types
    let types = serde_json::json!({
        "Certificate": [
            { "name": "name", "type": "string" },
            { "name": "uniqueId", "type": "string" },
            { "name": "serial", "type": "string" },
            { "name": "date", "type": "uint256" },
            { "name": "owner", "type": "address" },
            { "name": "metadataHash", "type": "bytes32" }
        ]
    });

    let metadata_hash = to_meta_hash(&certificate.metadata);
    // Create EIP-712 value
    let value = serde_json::json!({
        "name": certificate.name,
        "uniqueId": certificate.unique_id,
        "serial": certificate.serial,
        "date": certificate.date.to_string(),
        "owner": ToHexExt::encode_hex_upper_with_prefix(&certificate.owner),
        "metadataHash": Bytes::from(metadata_hash.to_vec()),
    });

    let eip712_object = Eip712Object {
        domain: custom_domain,
        types,
        value,
    };

    eprintln!("EIP-712 object created: {:?}", eip712_object);
    Ok(Json(eip712_object))
}
