use penumbra_crypto::asset;

pub fn balance_for_asset(asset_id: asset::Id) -> String {
    format!("dao/asset/{asset_id}")
}

pub fn all_assets() -> &'static str {
    // Note: this must be the prefix of the above.
    "dao/asset/"
}
