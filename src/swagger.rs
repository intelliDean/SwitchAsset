use crate::contract_calls::{
    get_all_contract_assets::__path_get_all_contract_assets, get_asset::__path_get_asset,
    get_my_assets::__path_get_my_assets, register_asset::__path_register_asset,
    transfer_asset::__path_transfer_asset,
};

use crate::handlers::{
        assets::__path_get_all_assets,
        search::__path_search_events,
        analytics::__path_get_analytics,
        transfer::{
        __path_get_assets_by_owner,
        __path_get_transfers_by_asset,
        __path_get_transfers_by_date
    }
};
use crate::models::{
    ApiResponse, Asset, GetAssetInput, RegisterAssetInput, Transfer, TransferAssetInput,
    OwnershipTransferredResponse,
    SearchInput,
    TransferByDate,
};
use utoipa::OpenApi;


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
        search_events,
        get_transfers_by_date,
        get_analytics
    ),
    components(
        schemas(
            Asset,
            Transfer,
            ApiResponse<Vec<Asset>>,
            ApiResponse<Vec<Transfer>>,
            RegisterAssetInput,
            GetAssetInput,
            TransferAssetInput,
            ApiResponse<Asset>,
            OwnershipTransferredResponse,
            SearchInput,
            TransferByDate
        )
    ),
    tags(
        (name = "SwitchAssets", description = "API for managing blockchain assets")
    )
)]
pub struct ApiDoc;