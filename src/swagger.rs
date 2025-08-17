use utoipa::OpenApi;
use crate::models::{Asset, Transfer, ApiResponse};
use crate::handlers::{assets::get_all_assets, transfer::{get_transfers_by_asset, get_assets_by_owner}};
use crate::handlers::assets::__path_get_all_assets;
use crate::handlers::transfer::{__path_get_transfers_by_asset, __path_get_assets_by_owner};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_all_assets,
        get_transfers_by_asset,
        get_assets_by_owner
    ),
    components(
        schemas(Asset, Transfer, ApiResponse<Vec<Asset>>, ApiResponse<Vec<Transfer>>)
    ),
    tags(
        (name = "SwitchAssets", description = "API for managing blockchain assets")
    )
)]
pub struct ApiDoc;