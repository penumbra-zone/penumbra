pub fn app_state() -> &'static str {
    "genesis/app_state"
}

/// This is used for delivering transactions originating from passed DaoSpend proposals; it lives in
/// the object store, not the consensus state.
pub fn synthetic_transactions() -> &'static str {
    "synthetic_transactions"
}
