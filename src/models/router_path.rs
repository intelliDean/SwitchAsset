use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RouterPath {
    pub  generate_signature: String,
    pub verify_authenticity: String,
    pub sign_up: String,
    pub get_owner: String,
    pub verify_signature: String,
    pub create_certificate: String,
    pub qr_code: String
}

impl RouterPath {
    pub fn init() -> Self {
        Self {
            generate_signature: "/generate_signature".to_string(),
            verify_authenticity: "/verify_authenticity".to_string(),
            sign_up: "/manufacturer_registers".to_string(),
            get_owner: "/get_owner/{address}".to_string(),
            verify_signature: "/verify_signature".to_string(),
            create_certificate: "/create_certificate".to_string(),
            qr_code: "/qr_code".to_string(),
        }
    }
}