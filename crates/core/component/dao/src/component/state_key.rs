use penumbra_asset::asset;

pub fn dao_params() -> &'static str {
    "dao/params"
}

pub fn dao_params_updated() -> &'static str {
    "dao/dao_params_updated"
}

pub fn balance_for_asset(asset_id: asset::Id) -> String {
    format!("dao/asset/{asset_id}")
}

pub fn all_assets_balance() -> &'static str {
    // note: this must be the prefix of the above.
    "dao/asset/"
}
