use penumbra_asset::asset;

pub fn balance_for_asset(asset_id: asset::Id) -> String {
    format!("dao/asset/{asset_id}")
}

pub fn all_assets_balance() -> &'static str {
    // Note: this must be the prefix of the above.
    "dao/asset/"
}
