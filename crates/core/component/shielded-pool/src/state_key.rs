pub mod denom_metadata_by_asset {
    use penumbra_sdk_asset::asset;
    use std::string::String;

    pub fn prefix() -> &'static str {
        "shielded_pool/assets/"
    }

    pub fn by_asset_id(asset_id: &asset::Id) -> String {
        format!("shielded_pool/assets/{asset_id}/denom")
    }
}

// State keys used to temporarily store payloads and nullifiers to be inserted into the compact
// block
pub fn pending_notes() -> &'static str {
    "shielded_pool/pending_notes"
}

pub fn pending_rolled_up_payloads() -> &'static str {
    "shielded_pool/pending_rolled_up_payloads"
}

pub fn shielded_pool_params() -> &'static str {
    "shielded_pool/params"
}
