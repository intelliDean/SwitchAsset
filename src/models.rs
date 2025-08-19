use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use diesel::{AsChangeset, Insertable, Queryable};
use ethabi::ethereum_types::{H160, H256};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::app_state::AppState;

#[derive(Queryable, Insertable, AsChangeset, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::assets)]
pub struct Asset {
    pub asset_id: String,
    pub owner: String,
    pub description: String,
    pub registered_at: i64,
}

#[derive(Queryable, Insertable, AsChangeset, Serialize, ToSchema)]
#[diesel(table_name = crate::schema::transfers)]
pub struct Transfer {
    pub id: i32,
    pub asset_id: String,
    pub old_owner: String,
    pub new_owner: String,
    pub timestamp: i64,
}


#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterAssetInput {
    pub description: String,
}



#[derive(Deserialize, ToSchema)]
pub struct TransferAssetInput {
    pub asset_id: String,
    pub new_owner: String,
}


#[derive(Serialize, ToSchema)]
pub struct OwnershipTransferredResponse {
    pub asset_id: String,
    pub old_owner: String,
    pub new_owner: String,
}

impl OwnershipTransferredResponse {
    pub fn init() -> Self {
        Self {
            asset_id: format!("0x{}", hex::encode(H256::zero())),
            old_owner: format!("0x{}", hex::encode(H160::zero())),
            new_owner: format!("0x{}", hex::encode(H160::zero())),
        }
    }

    pub fn new(asset_id: H256, old_owner: H160, new_owner: H160) -> Self {
        Self {
            asset_id: format!("0x{}", hex::encode(asset_id)),
            old_owner: format!("0x{}", hex::encode(old_owner.0)),
            new_owner: format!("0x{}", hex::encode(new_owner.0)),
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct GetAssetInput {
    pub asset_id: String,
}

// Structs for event responses
#[derive(Clone, Debug)]
pub struct AssetRegisteredResponse {
    pub asset_id: H256,
    pub asset_owner: H160,
}

impl AssetRegisteredResponse {
    pub fn init() -> Self {
        Self {
            asset_id: H256::zero(),
            asset_owner: H160::zero(),
        }
    }

    pub fn new(asset_id: H256, asset_owner: H160) -> Self {
        Self { asset_id, asset_owner }
    }
}






