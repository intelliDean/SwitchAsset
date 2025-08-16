use axum::http::StatusCode;
use axum::Json;
use qrcode::{EcLevel, QrCode};
use qrcode::render::svg;
use validator::Validate;
use crate::models::certificate_model::SignedCertificate;

#[utoipa::path(
    post,
    path = "/qr_code",
    request_body = SignedCertificate,
    responses(
        (status = 200, description = "QR code generated successfully", body = String),
        (status = 400, description = "Invalid input", body = String),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn generate_qr_code(
    Json(cert): Json<SignedCertificate>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    // to validate input
    if let Err(errors) = cert.validate() {
        return Err((StatusCode::BAD_REQUEST, errors.to_string()));
    }

    let cert_str = serde_json::to_string(&cert)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // check QR code size limit
    if cert_str.len() > 2953 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Certificate data too large for QR code".to_string(),
        ));
    }

    // EcLevel means medium error correction
    let qr_code = QrCode::with_error_correction_level(cert_str.as_bytes(), EcLevel::M)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // render as SVG
    let svg = qr_code
        .render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();

    Ok((StatusCode::OK, svg))
}