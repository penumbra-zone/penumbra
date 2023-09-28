/// Dao parameter data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DaoParameters {
    /// Whether DAO spend proposals are enabled.
    #[prost(bool, tag = "1")]
    pub dao_spend_proposals_enabled: bool,
}
/// Dao genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// Dao parameters.
    #[prost(message, optional, tag = "1")]
    pub dao_params: ::core::option::Option<DaoParameters>,
}
