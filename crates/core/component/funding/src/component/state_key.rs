pub fn staking_funding_parameters() -> &'static str {
    "funding/parameters"
}

pub mod lqt {
    pub mod v1 {
        pub mod nullifier {
            use penumbra_sdk_sct::Nullifier;

            pub(crate) fn lqt_nullifier_lookup_for_txid(
                epoch_index: u64,
                nullifier: &Nullifier,
            ) -> String {
                format!("funding/lqt/v1/nullifier/{epoch_index:020}/lookup/{nullifier}")
            }
        }
    }
}
