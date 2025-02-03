// The amount of staking tokens issued for this epoch.
pub fn staking_token_issuance_for_epoch() -> &'static str {
    "distributions/staking_token_issuance_for_epoch"
}

pub mod lqt {
    pub mod v1 {
        pub mod budget {
            pub(crate) fn prefix(epoch_index: u64) -> String {
                format!("distributions/lqt/v1/budget/{epoch_index:020}")
            }

            /// The amount of LQT rewards issued for this epoch.
            pub fn for_epoch(epoch_index: u64) -> [u8; 48] {
                let prefix_bytes = prefix(epoch_index);
                let mut key = [0u8; 48];
                key[0..48].copy_from_slice(prefix_bytes.as_bytes());
                key
            }
        }
    }
}

pub fn distributions_parameters() -> &'static str {
    "distributions/parameters"
}
