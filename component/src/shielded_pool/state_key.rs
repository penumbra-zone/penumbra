use jmt::KeyHash;
use penumbra_crypto::{asset, note, Nullifier};
use penumbra_tct::{
    builder::{block, epoch},
    Root,
};

pub fn token_supply(asset_id: &asset::Id) -> KeyHash {
    format!("shielded_pool/assets/{}/token_supply", asset_id).into()
}

pub fn known_assets() -> KeyHash {
    "shielded_pool/known_assets".into()
}

pub fn denom_by_asset(asset_id: &asset::Id) -> KeyHash {
    format!("shielded_pool/assets/{}/denom", asset_id).into()
}

pub fn note_source(note_commitment: note::Commitment) -> KeyHash {
    format!("shielded_pool/note_source/{}", note_commitment).into()
}

pub fn compact_block(height: u64) -> KeyHash {
    format!("shielded_pool/compact_block/{}", height).into()
}

pub fn anchor_by_height(height: u64) -> KeyHash {
    format!("shielded_pool/anchor/{}", height).into()
}

pub fn anchor_lookup(anchor: Root) -> KeyHash {
    format!("shielded_pool/valid_anchors/{}", anchor).into()
}

pub fn epoch_anchor_by_index(index: u64) -> KeyHash {
    format!("shielded_pool/epoch_anchor/{}", index).into()
}

pub fn epoch_anchor_lookup(anchor: epoch::Root) -> KeyHash {
    format!("shielded_pool/valid_epoch_anchors/{}", anchor).into()
}

pub fn block_anchor_by_height(height: u64) -> KeyHash {
    format!("shielded_pool/block_anchor/{}", height).into()
}

pub fn block_anchor_lookup(anchor: block::Root) -> KeyHash {
    format!("shielded_pool/valid_block_anchors/{}", anchor).into()
}

pub fn spent_nullifier_lookup(nullifier: Nullifier) -> KeyHash {
    format!("shielded_pool/spent_nullifiers/{}", nullifier).into()
}

pub fn commission_amounts(height: u64) -> KeyHash {
    format!("staking/commission_amounts/{}", height).into()
}

pub fn scheduled_to_apply(epoch: u64) -> KeyHash {
    format!("shielded_pool/quarantined_to_apply_in_epoch/{}", epoch).into()
}

pub fn quarantined_spent_nullifier_lookup(nullifier: Nullifier) -> KeyHash {
    format!("shielded_pool/quarantined_spent_nullifiers/{}", nullifier).into()
}

pub use crate::stake::state_key::slashed_validators;
