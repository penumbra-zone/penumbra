/// State key for the asset regulation tree
pub fn asset_tree() -> &'static str {
    "compliance/asset_tree"
}

/// State key for the user compliance tree
pub fn user_tree() -> &'static str {
    "compliance/user_tree"
}

/// State key for the user count (number of registered users)
pub fn user_count() -> &'static str {
    "compliance/user_count"
}

/// State key for the asset count (number of registered assets)
pub fn asset_count() -> &'static str {
    "compliance/asset_count"
}

/// State key for mapping an asset ID to its position in the asset tree
pub fn asset_index(asset_id: &penumbra_sdk_asset::asset::Id) -> String {
    format!("compliance/asset_index/{}", asset_id)
}

/// State key for the public regulation status of an asset
pub fn asset_status(asset_id: &penumbra_sdk_asset::asset::Id) -> String {
    format!("compliance/asset_status/{}", asset_id)
}

/// State key for reverse lookup: (address, asset_id) -> position in user tree
/// This enables O(1) lookup of a user's leaf position for merkle path generation
pub fn user_leaf_position(
    address: &penumbra_sdk_keys::Address,
    asset_id: &penumbra_sdk_asset::asset::Id,
) -> String {
    format!("compliance/user_lookup/{}/{}", address, asset_id)
}

/// State key for storing the full ComplianceLeaf data for a user
/// This allows retrieving the complete leaf (including ACK) for proof generation
pub fn user_leaf_data(
    address: &penumbra_sdk_keys::Address,
    asset_id: &penumbra_sdk_asset::asset::Id,
) -> String {
    format!("compliance/user_leaf/{}/{}", address, asset_id)
}
