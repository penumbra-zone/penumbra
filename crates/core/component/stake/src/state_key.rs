use std::string::String;
use tendermint::PublicKey;

use crate::IdentityKey;

pub fn stake_params() -> &'static str {
    "staking/params"
}

pub fn stake_params_updated() -> &'static str {
    "staking/stake_params_updated"
}

pub fn current_base_rate() -> &'static str {
    "staking/base_rate/current"
}

pub fn previous_base_rate() -> &'static str {
    "staking/base_rate/previous"
}

// TODO(erwan): refactor the state key so that each module
// represents a coherent set of keys, so that the module path
// aligns roughly with the actual schema of the data.
// Tracked by #3615.
pub mod validators {
    pub mod index {
        pub mod consensus_set {
            pub fn prefix() -> &'static str {
                "staking/validators/index/consensus_set/"
            }
            pub fn by_id(id: &crate::IdentityKey) -> String {
                format!("{}{id}", prefix())
            }
        }
    }

    pub mod definitions {
        pub fn prefix() -> &'static str {
            "staking/validators/definitions/"
        }
        pub fn by_id(id: &crate::IdentityKey) -> String {
            format!("{}{id}", prefix())
        }
    }
}

pub fn penalty_in_epoch(id: &IdentityKey, epoch: u64) -> String {
    // Load-bearing format string: we need to pad with 0s to ensure that
    // the lex order agrees with the numeric order on epochs.
    // 10 decimal digits covers 2^32 epochs.
    format!("staking/penalty_in_epoch/{id}/{epoch:010}")
}

pub fn penalty_in_epoch_prefix(id: &IdentityKey) -> String {
    format!("staking/penalty_in_epoch/{id}/")
}

pub fn state_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator_state/{id}")
}

pub fn current_rate_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator_rate/current/{id}")
}

pub fn next_rate_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator_rate/next/{id}")
}

pub fn power_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator_power/{id}")
}

pub fn bonding_state_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator_bonding_state/{id}")
}

pub fn uptime_by_validator(id: &IdentityKey) -> String {
    format!("staking/validator_uptime/{id}")
}

pub fn slashed_validators(height: u64) -> String {
    format!("staking/slashed_validators/{height}")
}

pub fn validator_id_by_consensus_key(pk: &PublicKey) -> String {
    format!("staking/validator_id_by_consensus_key/{}", pk.to_hex())
}

pub fn consensus_key_by_tendermint_address(address: &[u8; 20]) -> String {
    format!(
        "staking/consensus_key_by_tendermint_address/{}",
        hex::encode(address)
    )
}

pub fn delegation_changes_by_height(height: u64) -> String {
    format!("staking/delegation_changes/{height}")
}

pub fn current_consensus_keys() -> &'static str {
    "staking/current_consensus_keys"
}

pub(super) mod internal {
    pub fn delegation_changes() -> &'static str {
        "staking/delegation_changes"
    }

    pub fn tendermint_validator_updates() -> &'static str {
        "staking/tendermint_validator_updates"
    }
}

#[cfg(test)]
mod tests {
    use decaf377_rdsa as rdsa;
    use std::collections::BTreeSet;

    use super::*;
    use rand_core::OsRng;

    #[test]
    fn penalty_in_epoch_padding() {
        let sk = rdsa::SigningKey::new(OsRng);
        let ik = IdentityKey((&sk).into());

        assert_eq!(
            penalty_in_epoch(&ik, 791),
            //                                     0123456789
            format!("staking/penalty_in_epoch/{ik}/0000000791"),
        );
    }

    #[test]
    fn penalty_in_epoch_sorting() {
        let sk = rdsa::SigningKey::new(OsRng);
        let ik = IdentityKey((&sk).into());

        let k791 = penalty_in_epoch(&ik, 791);
        let k792 = penalty_in_epoch(&ik, 792);
        let k793 = penalty_in_epoch(&ik, 793);
        let k79 = penalty_in_epoch(&ik, 79);
        let k7 = penalty_in_epoch(&ik, 7);

        let keys = vec![k791.clone(), k792.clone(), k793.clone(), k79, k7]
            .into_iter()
            .collect::<BTreeSet<String>>();

        // All keys are distinct
        assert_eq!(keys.len(), 5);

        // Check that lex order agrees with numeric order
        let range = keys
            .range(k791.clone()..=k793.clone())
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(range, vec![k791, k792, k793,]);
    }
}
