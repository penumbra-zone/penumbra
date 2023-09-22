/// Dao configuration data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DaoParameters {
    /// Whether DAO spend proposals are enabled.
    #[prost(bool, tag = "1")]
    pub dao_spend_proposals_enabled: bool,
}
