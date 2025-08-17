use diesel::Queryable;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Queryable, Serialize, ToSchema)]
pub struct Asset {
    pub asset_id: String,
    pub owner: String,
    pub description: String,
    pub registered_at: i64,
}

#[derive(Queryable, Serialize, ToSchema)]
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


