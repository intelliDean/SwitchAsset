use crate::contract_calls::{
    __path_get_all_contract_assets, __path_get_asset,
    __path_get_my_assets, __path_register_asset, __path_transfer_asset,
};
use crate::contract_calls::{
    get_all_contract_assets, get_asset, get_my_assets, register_asset,
    transfer_asset,
};
use crate::handlers::assets::__path_get_all_assets;
use crate::handlers::transfer::{__path_get_assets_by_owner, __path_get_transfers_by_asset};
use crate::handlers::{
    assets::get_all_assets,
    transfer::{get_assets_by_owner, get_transfers_by_asset},
};
use crate::models::{
    ApiResponse, Asset, GetAssetInput, RegisterAssetInput, Transfer, TransferAssetInput,
};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_assets,
        get_transfers_by_asset,
        get_assets_by_owner,
        get_asset,
        transfer_asset,
        register_asset,
        get_my_assets,
        get_all_contract_assets,
    ),
    components(
        schemas(
            Asset, Transfer, ApiResponse<Vec<Asset>>, ApiResponse<Vec<Transfer>>,
            RegisterAssetInput, GetAssetInput, TransferAssetInput, ApiResponse<Asset>
        )
    ),
    tags(
        (name = "SwitchAssets", description = "API for managing blockchain assets")
    )
)]
pub struct ApiDoc;
