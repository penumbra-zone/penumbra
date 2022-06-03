use jmt::KeyHash;
use penumbra_crypto::{asset, note, IdentityKey, Nullifier};
use penumbra_tct::Root;

pub fn token_supply(asset_id: &asset::Id) -> KeyHash {
    format!("shielded_pool/assets/{}/token_supply", asset_id).into()
}

pub fn known_assets() -> KeyHash {
    "shielded_pool/known_assets".into()
}

pub fn denom_by_asset(asset_id: &asset::Id) -> KeyHash {
    format!("shielded_pool/assets/{}/denom", asset_id).into()
}

pub fn note_source(note_commitment: &note::Commitment) -> KeyHash {
    format!("shielded_pool/note_source/{}", note_commitment).into()
}

pub fn compact_block(height: u64) -> KeyHash {
    format!("shielded_pool/compact_block/{}", height).into()
}

pub fn anchor_by_height(height: &u64) -> KeyHash {
    format!("shielded_pool/anchor/{}", height).into()
}

pub fn anchor_lookup(anchor: &Root) -> KeyHash {
    format!("shielded_pool/valid_anchors/{}", anchor).into()
}

pub fn spent_nullifier_lookup(nullifier: &Nullifier) -> KeyHash {
    format!("shielded_pool/spent_nullifiers/{}", nullifier).into()
}

pub fn commission_amounts(height: u64) -> KeyHash {
    format!("staking/commission_amounts/{}", height).into()
}

pub fn quarantined_note_source(note_commitment: &note::Commitment) -> KeyHash {
    format!("shielded_pool/quarantined_note_source/{}", note_commitment).into()
}

pub fn quarantined_notes_to_apply_at_height(height: u64) -> KeyHash {
    format!(
        "shielded_pool/quarantined_notes_to_apply_at_height/{}",
        height
    )
    .into()
}

pub fn quarantined_nullifiers_to_apply_at(height: u64) -> KeyHash {
    format!(
        "shielded_pool/quarantined_nullifiers_to_apply_at_height/{}",
        height
    )
    .into()
}

// NOTE: Quarantined notes and nullifiers mapped to validators are keyed by height as well as
// validator identity key. This means that is more expensive to look up all the relevant quarantined
// notes/nullifiers for a validator when slashing (you need to do a manual range iteration over all
// the heights from now back through the preceding unbonding period), but it means that it's faster
// to add more quarantined notes/nullifiers to a validator, which needs to happen in every block. Sot
// rather than reading a single key with a huge value (all the quarantined notes/nullifiers for the
// preceding unbonding period) every single block, we write a smaller value (only the
// notes/nullifiers that were quarantined in this block), and read a whole range of keys, **but
// only** when slashing.

pub fn quarantined_notes_connected_to_validator_at(
    validator_identity: IdentityKey,
    height: u64,
) -> KeyHash {
    format!(
        "shielded_pool/quarantined_notes_connected_to_validator_at/{}/{}",
        validator_identity, height
    )
    .into()
}

pub fn quarantined_nullifiers_connected_to_validator_at(
    validator_identity: IdentityKey,
    height: u64,
) -> KeyHash {
    format!(
        "shielded_pool/quarantined_nullifiers_connected_to_validator_at/{}/{}",
        validator_identity, height
    )
    .into()
}

pub fn quarantined_spent_nullifier_lookup(nullifier: &Nullifier) -> KeyHash {
    format!("shielded_pool/quarantined_spent_nullifiers/{}", nullifier).into()
}
