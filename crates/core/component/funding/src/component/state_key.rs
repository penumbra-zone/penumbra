pub fn funding_parameters() -> &'static str {
    "funding/parameters"
}

pub mod lqt {
    pub mod v1 {
        pub mod nullifier {
            use penumbra_sdk_sct::Nullifier;

            /// A nullifier set indexed by epoch, mapping each epoch to its corresponding `TransactionId`.
            pub(crate) fn key(epoch_index: u64, nullifier: &Nullifier) -> String {
                format!("funding/lqt/v1/nullifier/{epoch_index:020}/lookup/{nullifier}")
            }
        }
    }
}
