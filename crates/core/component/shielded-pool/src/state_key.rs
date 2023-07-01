use penumbra_asset::asset;
use penumbra_crypto::Nullifier;
use std::string::String;

pub fn token_supply(asset_id: &asset::Id) -> String {
    format!("shielded_pool/assets/{asset_id}/token_supply")
}

pub fn known_assets() -> &'static str {
    "shielded_pool/known_assets"
}

pub fn denom_by_asset(asset_id: &asset::Id) -> String {
    format!("shielded_pool/assets/{asset_id}/denom")
}

pub fn spent_nullifier_lookup(nullifier: &Nullifier) -> String {
    format!("shielded_pool/spent_nullifiers/{nullifier}")
}

// State keys used to temporarily store payloads and nullifiers to be inserted into the compact
// block

pub fn pending_notes() -> &'static str {
    "shielded_pool/pending_notes"
}

pub fn pending_rolled_up_notes() -> &'static str {
    "shielded_pool/pending_rolled_up_notes"
}

pub fn pending_nullifiers() -> &'static str {
    "shielded_pool/pending_nullifiers"
}
