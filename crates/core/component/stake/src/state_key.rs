pub mod parameters {
    pub fn key() -> &'static str {
        "staking/parameters"
    }
}

pub mod validators {
    pub mod consensus_set_index {
        pub fn prefix() -> &'static str {
            "staking/validators/consensus_set_index/"
        }
        pub fn by_id(id: &crate::IdentityKey) -> String {
            format!("{}{id}", prefix())
        }
    }

    pub mod lookup_by {
        use tendermint::PublicKey;

        pub fn consensus_key(pk: &PublicKey) -> String {
            format!("staking/validators/lookup_by/consensus_key/{}", pk.to_hex())
        }

        pub fn cometbft_address(address: &[u8; 20]) -> String {
            format!(
                "staking/validators/lookup_by/cometbft_address/{}",
                hex::encode(address)
            )
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

    pub mod state {
        pub fn by_id(id: &crate::IdentityKey) -> String {
            format!("staking/validators/data/state/{id}")
        }
    }

    pub mod rate {
        pub fn current_by_id(id: &crate::IdentityKey) -> String {
            format!("staking/validators/data/rate/current/{id}")
        }

        pub fn previous_by_id(id: &crate::IdentityKey) -> String {
            format!("staking/validators/data/rate/previous/{id}")
        }
    }

    pub mod power {
        pub fn by_id(id: &crate::IdentityKey) -> String {
            format!("staking/validators/data/power/{id}")
        }
    }

    pub mod pool {
        pub mod balance {
            pub fn by_id(id: &crate::IdentityKey) -> String {
                format!("staking/validators/data/pool/balance/{id}")
            }
        }

        pub mod bonding_state {
            pub fn by_id(id: &crate::IdentityKey) -> String {
                format!("staking/validators/data/pool/bonding_state/{id}")
            }
        }
    }

    pub mod uptime {
        pub fn by_id(id: &crate::IdentityKey) -> String {
            format!("staking/validators/data/uptime/{id}")
        }
    }

    pub mod last_disabled {
        pub fn by_id(id: &crate::IdentityKey) -> String {
            format!("staking/validators/data/last_disabled/{id}")
        }
    }

    /// Tracks the funding rewards of the previously active validator set
    /// in object storage. Consumed by the funding component.
    pub mod rewards {
        pub fn staking() -> &'static str {
            "staking/validators/staking_rewards"
        }
    }
}

pub mod chain {
    pub mod base_rate {
        pub fn current() -> &'static str {
            "staking/chain/base_rate/current"
        }

        pub fn previous() -> &'static str {
            "staking/chain/base_rate/previous"
        }
    }

    pub fn total_bonded() -> &'static str {
        "staking/chain/total_bonded"
    }

    pub mod delegation_changes {
        pub fn key() -> &'static str {
            "staking/delegation_changes"
        }

        pub fn by_height(height: u64) -> String {
            format!("staking/delegation_changes/{height}")
        }
    }
}

pub mod penalty {
    use crate::IdentityKey;

    pub fn prefix(id: &IdentityKey) -> String {
        // Note: We typically put the key at the end of the path to increase
        // locality. Here we don't because we want to build a prefix iterator
        // to accumulate validator penalty across epochs.
        format!("staking/penalty/{id}/")
    }
    pub fn for_id_in_epoch(id: &crate::IdentityKey, epoch_index: u64) -> String {
        // Load-bearing format string: we need to pad with 0s to ensure that
        // the lex order agrees with the numeric order on epochs.
        // 10 decimal digits covers 2^32 epochs.
        format!("{}{epoch_index:010}", prefix(id))
    }
}

pub mod consensus_update {
    pub fn consensus_keys() -> &'static str {
        "staking/cometbft_data/consensus_keys"
    }
}

pub(super) mod internal {

    pub fn cometbft_validator_updates() -> &'static str {
        "staking/cometbft_validator_updates"
    }
}

#[cfg(test)]
mod tests {
    use decaf377_rdsa as rdsa;
    use std::collections::BTreeSet;
    use tests::penalty;

    use crate::IdentityKey;

    use super::*;
    use rand_core::OsRng;

    #[test]
    fn penalty_in_epoch_padding() {
        let vk = rdsa::VerificationKey::from(rdsa::SigningKey::new(OsRng));
        let ik = IdentityKey(vk.into());

        assert_eq!(
            penalty::for_id_in_epoch(&ik, 791),
            //                            0123456789
            format!("staking/penalty/{ik}/0000000791")
        );
    }

    #[test]
    fn penalty_in_epoch_sorting() {
        let vk = rdsa::VerificationKey::from(rdsa::SigningKey::new(OsRng));
        let ik = IdentityKey(vk.into());

        let k791 = penalty::for_id_in_epoch(&ik, 791);
        let k792 = penalty::for_id_in_epoch(&ik, 792);
        let k793 = penalty::for_id_in_epoch(&ik, 793);
        let k79 = penalty::for_id_in_epoch(&ik, 79);
        let k7 = penalty::for_id_in_epoch(&ik, 7);

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
