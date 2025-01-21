use penumbra_sdk_asset::asset;

pub fn community_pool_params() -> &'static str {
    "community_pool/params"
}

pub fn balance_for_asset(asset_id: asset::Id) -> String {
    format!("community_pool/asset/{asset_id}")
}

pub fn all_assets_balance() -> &'static str {
    // note: this must be the prefix of the above.
    "community_pool/asset/"
}
