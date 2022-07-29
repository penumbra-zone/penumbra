use penumbra_crypto::IdentityKey;
use std::string::String;
use tendermint::PublicKey;

pub fn validator_list() -> &'static str {
    "staking/validators"
}

pub fn current_base_rate() -> &'static str {
    "staking/base_rate/current"
}

pub fn next_base_rate() -> &'static str {
    "staking/base_rate/next"
}

pub fn validator_by_id(id: &IdentityKey) -> String {
    format!("staking/validator/{}", id)
}

pub fn state_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator/{}/state", id)
}

pub fn current_rate_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator/{}/rate/current", id)
}

pub fn next_rate_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator/{}/rate/next", id)
}

pub fn power_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator/{}/power", id)
}

pub fn bonding_state_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator/{}/bonding_state", id)
}

pub fn uptime_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator/{}/uptime", id)
}

pub fn slashed_validators(height: u64) -> String {
    format!("staking/slashed_validators/{}", height)
}

pub fn validator_id_by_consensus_key(pk: &PublicKey) -> String {
    format!("staking/validator_id_by_consensus_key/{}", pk.to_hex())
}

pub fn delegation_changes_by_height(height: u64) -> String {
    format!("staking/delegation_changes/{}", height)
}
