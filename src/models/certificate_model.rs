use crate::config::app_router::authenticity;
use crate::utility::to_meta_hash;
use ethabi::ethereum_types::{Address, U256};
use ethers::contract::EthEvent;
use ethers::types::transaction::eip712::{EIP712Domain, Eip712, Eip712Error};
use ethers::utils::keccak256;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::env;
use ethabi::Bytes;
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

// Certificate struct for EIP-712
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Certificate {
    pub name: String,
    pub unique_id: String,
    pub serial: String,
    pub date: U256,
    pub owner: Address,
    pub metadata_hash: [u8; 32],
    pub metadata: Vec<String>,
}

// EIP-712 implementation
impl Eip712 for Certificate {
    type Error = Eip712Error;

    fn domain_separator(&self) -> Result<[u8; 32], Self::Error> {
        let domain = self.domain()?;
        let type_hash = keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );

        let name_hash = keccak256(domain.name.unwrap_or_default().as_bytes());
        let version_hash = keccak256(domain.version.unwrap_or_default().as_bytes());
        let chain_id = U256::from(domain.chain_id.unwrap_or_default());
        let verifying_contract = domain.verifying_contract.unwrap_or_default();

        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(type_hash.to_vec()),
            ethers::abi::Token::FixedBytes(name_hash.to_vec()),
            ethers::abi::Token::FixedBytes(version_hash.to_vec()),
            ethers::abi::Token::Uint(chain_id),
            ethers::abi::Token::Address(verifying_contract),
        ]);
        Ok(keccak256(&encoded))
    }
    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        let factory_address: Address = env::var("CONTRACT_ADDRESS")
            .expect("CONTRACT ADDRESS NOT SET")
            .parse()
            .expect("Invalid contract address");

        let chain_id = env::var("CHAIN_ID").unwrap().parse::<usize>().unwrap();

        Ok(EIP712Domain {
            // name: Some("CertificateAuth".to_string()),
            name: Some(env::var("SIGNING_DOMAIN").unwrap()),
            // version: Some("1".to_string()),
            version: Some(env::var("SIGNATURE_VERSION").unwrap()),
            chain_id: Some(U256::from(chain_id).into()),
            verifying_contract: Some(factory_address),
            salt: None,
        })
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        
        // let certificate = env::var("CERTIFICATE").unwrap();
        
        Ok(keccak256( //i will add it to the env file
                      env::var("CERTIFICATE").unwrap(),     //"Certificate(string name,string uniqueId,string serial,uint256 date,address owner,bytes32 metadataHash)",
        ))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {

        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(Self::type_hash()?.to_vec()),
            ethers::abi::Token::FixedBytes(keccak256(self.name.as_bytes()).to_vec()),
            ethers::abi::Token::FixedBytes(keccak256(self.unique_id.as_bytes()).to_vec()),
            ethers::abi::Token::FixedBytes(keccak256(self.serial.as_bytes()).to_vec()),
            ethers::abi::Token::Uint(self.date),
            ethers::abi::Token::Address(self.owner),
            ethers::abi::Token::FixedBytes(self.metadata_hash.to_vec()),
        ]);
        
        Ok(keccak256(&encoded))
    }

    fn encode_eip712(&self) -> Result<[u8; 32], Self::Error> {
        let domain_separator = self.domain_separator()?;
        let struct_hash = self.struct_hash()?;

        let mut bytes = Vec::with_capacity(2 + 32 + 32);
        bytes.extend_from_slice(b"\x19\x01"); //this is adding the \x19Ethereum Signed Message prefix
        bytes.extend_from_slice(&domain_separator);
        bytes.extend_from_slice(&struct_hash);

        Ok(keccak256(&bytes))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, ToSchema, Validate)]
pub struct SignedCertificate {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub unique_id: String,
    #[validate(length(min = 1))]
    pub serial: String,
    pub date: u64,
    #[validate(custom(function = "validate_address"))]
    #[schema(value_type = String, format = Binary)]
    pub owner: String, 
    #[validate(length(min = 1))]
    pub metadata: Vec<String>,
    #[validate(custom(function = "validate_signature"))]
    #[schema(value_type = String, format = Binary)]
    pub signature: String,
}

fn validate_address(address: &String) -> Result<(), ValidationError> {
    if !address.starts_with("0x") || address.len() != 42 || hex::decode(&address[2..]).is_err() {
        return Err(ValidationError::new("Invalid Ethereum address"));
    }
    Ok(())
}
fn validate_signature(signature: &String) -> Result<(), ValidationError> {
    if !signature.starts_with("0x")
        || (signature.len() != 130 && signature.len() != 132)
        || hex::decode(&signature[2..]).is_err()
    {
        return Err(ValidationError::new("Invalid signature"));
    }
    Ok(())
}

impl TryFrom<SignedCertificate> for Certificate {
    type Error = anyhow::Error;
    fn try_from(dto: SignedCertificate) -> Result<Self, Self::Error> {
        
        Ok(Certificate {
            name: dto.name,
            unique_id: dto.unique_id,
            serial: dto.serial,
            date: U256::from(dto.date),
            owner: dto
                .owner
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid address format"))?,
            metadata_hash: to_meta_hash(&dto.metadata),
            metadata: dto.metadata,
        })
    }
}

impl From<Certificate> for authenticity::Certificate {
    fn from(cert: Certificate) -> Self {
        
        Self {
            name: cert.name,
            unique_id: cert.unique_id,
            serial: cert.serial,
            date: cert.date,
            owner: cert.owner,
            metadata: cert.metadata,
            metadata_hash: cert.metadata_hash
        }
    }
}

// Custom EIP712Domain for ToSchema
#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
pub struct CustomEIP712Domain {
    #[schema(value_type = String, nullable = true)]
    pub name: Option<String>,
    #[schema(value_type = String, nullable = true)]
    pub version: Option<String>,
    #[schema(value_type = String, nullable = true)]
    #[serde(rename = "chainId")]
    pub chain_id: Option<String>, // Serialize as chainId
    #[schema(value_type = String, nullable = true)]
    #[serde(rename = "verifyingContract")]
    pub verifying_contract: Option<String>, // Serialize as verifyingContract
    #[schema(value_type = String, nullable = true)]
    pub salt: Option<String>,
}

// Convert EIP712Domain to CustomEIP712Domain
impl From<EIP712Domain> for CustomEIP712Domain {
    fn from(domain: EIP712Domain) -> Self {
        CustomEIP712Domain {
            name: domain.name,
            version: domain.version,
            chain_id: domain.chain_id.map(|id| id.to_string()),
            verifying_contract: domain.verifying_contract.map(|addr| format!("{:?}", addr)),
            salt: domain.salt.map(|s| format!("{:?}", s)),
        }
    }
}

// EIP-712 object for frontend signing
#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
pub struct Eip712Object {
    pub domain: CustomEIP712Domain,
    pub types: serde_json::Value,
    pub value: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
pub struct CertificateData {
    pub name: String,
    pub unique_id: String,
    pub serial: String,
    pub date: u64,
    #[schema(value_type = String, format = Binary)]
    pub owner: String,
    pub metadata: Vec<String>,
}

impl TryFrom<CertificateData> for Certificate {
    type Error = anyhow::Error;
    fn try_from(dto: CertificateData) -> Result<Self, Self::Error> {
        
        Ok(Certificate {
            name: dto.name,
            unique_id: dto.unique_id,
            serial: dto.serial,
            date: U256::from(dto.date),
            owner: dto
                .owner
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid address format"))?,
            metadata_hash: to_meta_hash(&dto.metadata),
            metadata: dto.metadata,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
pub struct RegInput {
    pub name: String,
}

