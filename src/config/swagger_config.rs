use crate::services::other_tests::{
    __path_generate_signature, __path_manufacturer_registers, __path_get_owner, __path_verify_signature};
use crate::services::verify_authenticity::__path_verify_authenticity;
use crate::services::create_eip712::__path_create_certificate;
use crate::services::qr_code::__path_generate_qr_code;
use utoipa::OpenApi;
use crate::models::certificate_model::{RegInput, SignedCertificate, CertificateData, Eip712Object};

// Swagger/OpenAPI configuration
#[derive(OpenApi)]
#[openapi(
    paths(
        verify_authenticity,
        generate_signature,
        manufacturer_registers,
        get_owner,
        verify_signature,
        create_certificate,
        generate_qr_code
    ),
    components(
        schemas(RegInput, CertificateData, SignedCertificate, Eip712Object),
        // responses(Item)
    ),
    tags(
        (name = "ERI", description = "Signature Verifying APIs")
    ),
    info(
        title = "ERI APIs",
        description = "Signature Verifying Project on the Blockchain",
    ),


    // security(
    //     (),
    //     ("my_auth" = ["read:items", "edit:items"]),
    //     ("token_jwt" = [])
    // ),
    // servers(
    //     (url = "http://localhost:8989", description = "Local server"),
    //     (url = "http://api.{username}:{port}", description = "Remote API",
    //         variables(
    //             ("username" = (default = "demo", description = "Default username for API")),
    //             ("port" = (default = "8080", enum_values("8080", "5000", "3030"), description = "Supported ports for API"))
    //         )
    //     )
    // )
)]
pub struct ApiDoc;
