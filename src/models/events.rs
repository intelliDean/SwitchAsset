use ethabi::ethereum_types::{Address, H256};
use ethers::contract::EthEvent;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, EthEvent, Serialize, Deserialize)]
#[ethevent(name = "ManufacturerRegistered", abi = "ManufacturerRegistered(address,address)")]
#[derive(Default)]
pub struct ManufacturerRegistered {
    
    // pub name: String,

    #[ethevent(indexed)]
    pub manufacturer_address: Address,

    #[ethevent(indexed)]
    pub manufacturer_name: String,
}

impl ManufacturerRegistered {
    pub fn init() -> Self {
        Self {
            manufacturer_address: Address::zero(),
            manufacturer_name: String::new(),
        }
    }
    pub fn new(manufacturer_address: Address, manufacturer_name: String) -> Self {
        Self {
            manufacturer_address,
            manufacturer_name,
        }
    }
}
