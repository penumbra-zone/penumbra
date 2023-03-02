#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DaoDeposit {
    /// The value to deposit into the DAO.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DaoSpend {
    /// The value to spend from the DAO.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DaoOutput {
    /// The value to output from the DAO.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Value>,
    /// The address to send the output to.
    #[prost(message, optional, tag = "2")]
    pub address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
}
