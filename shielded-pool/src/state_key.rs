use jmt::KeyHash;
use penumbra_chain::CompactBlock;
use penumbra_crypto::{
    asset::{Denom, Id as Asset_Id},
    note, Nullifier,
};
use penumbra_tct::Root;

pub fn token_supply(asset_id: &Asset_Id) -> KeyHash {
    format!("shielded_pool/assets/{}/token_supply", asset_id).into()
}

pub fn known_assets() -> KeyHash {
    format!("shielded_pool/known_assets").into()
}

pub fn denom_by_asset(asset_id: &Asset_Id) -> KeyHash {
    format!("shielded_pool/assets/{}/denom", asset_id).into()
}

pub fn register_denom(denom: &Denom) -> KeyHash {
    format!("shielded_pool/assets/{}/denom", denom).into()
}

pub fn note_source(note_commitment: &note::Commitment) -> KeyHash {
    format!("shielded_pool/note_source/{}", note_commitment).into()
}

pub fn set_compact_block(compact_block: &CompactBlock) -> KeyHash {
    format!("shielded_pool/compact_block/{}", compact_block.height).into()
}

pub fn compact_block(height: u64) -> KeyHash {
    format!("shielded_pool/compact_block/{}", height).into()
}

pub fn nct_anchor_tct(nct_anchor: &Root) -> KeyHash {
    format!("shielded_pool/tct_anchor/{}", nct_anchor).into()
}

pub fn nct_anchor_valid(nct_anchor: &Root) -> KeyHash {
    format!("shielded_pool/valid_anchors/{}", nct_anchor).into()
}

pub fn claimed_anchor(anchor: &Root) -> KeyHash {
    format!("shielded_pool/valid_anchors/{}", anchor).into()
}

pub fn spend_nullifier(nullifier: &Nullifier) -> KeyHash {
    format!("shielded_pool/spent_nullifiers/{}", nullifier).into()
}

pub fn commission_amounts(height: u64) -> KeyHash {
    format!("staking/commission_amounts/{}", height).into()
}
