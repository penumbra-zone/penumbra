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

pub fn consensus_key_by_tendermint_address(address: &[u8; 20]) -> String {
    format!(
        "staking/consensus_key_by_tendermint_address/{}",
        hex::encode(&address)
    )
}

pub fn delegation_changes_by_height(height: u64) -> String {
    format!("staking/delegation_changes/{}", height)
}

pub fn current_consensus_keys() -> &'static str {
    "staking/current_consensus_keys"
}

pub(super) mod internal {
    pub fn stub_delegation_changes() -> &'static str {
        "staking/delegation_changes"
    }

    pub fn stub_tendermint_validator_updates() -> &'static str {
        "staking/tendermint_validator_updates"
    }
}