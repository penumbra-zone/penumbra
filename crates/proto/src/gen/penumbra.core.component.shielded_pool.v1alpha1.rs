#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Note {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub address: ::core::option::Option<super::super::super::keys::v1alpha1::Address>,
}
impl ::prost::Name for Note {
    const NAME: &'static str = "Note";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteView {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::ValueView>,
    #[prost(bytes = "vec", tag = "2")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub address: ::core::option::Option<
        super::super::super::keys::v1alpha1::AddressView,
    >,
}
impl ::prost::Name for NoteView {
    const NAME: &'static str = "NoteView";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// An encrypted note.
/// 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteCiphertext {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for NoteCiphertext {
    const NAME: &'static str = "NoteCiphertext";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// The body of an output description, including only the minimal
/// data required to scan and process the output.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotePayload {
    /// The note commitment for the output note. 32 bytes.
    #[prost(message, optional, tag = "1")]
    pub note_commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// The encoding of an ephemeral public key. 32 bytes.
    #[prost(bytes = "vec", tag = "2")]
    pub ephemeral_key: ::prost::alloc::vec::Vec<u8>,
    /// An encryption of the newly created note.
    /// 132 = 1(type) + 11(d) + 8(amount) + 32(asset_id) + 32(rcm) + 32(pk_d) + 16(MAC) bytes.
    #[prost(message, optional, tag = "3")]
    pub encrypted_note: ::core::option::Option<NoteCiphertext>,
}
impl ::prost::Name for NotePayload {
    const NAME: &'static str = "NotePayload";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// A Penumbra ZK output proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkOutputProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkOutputProof {
    const NAME: &'static str = "ZKOutputProof";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// A Penumbra ZK spend proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkSpendProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkSpendProof {
    const NAME: &'static str = "ZKSpendProof";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// A Penumbra ZK nullifier derivation proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkNullifierDerivationProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkNullifierDerivationProof {
    const NAME: &'static str = "ZKNullifierDerivationProof";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// Spends a shielded note.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Spend {
    /// The effecting data of the spend.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<SpendBody>,
    /// The authorizing signature for the spend.
    #[prost(message, optional, tag = "2")]
    pub auth_sig: ::core::option::Option<
        super::super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendAuthSignature,
    >,
    /// The proof that the spend is well-formed is authorizing data.
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<ZkSpendProof>,
}
impl ::prost::Name for Spend {
    const NAME: &'static str = "Spend";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// ABCI Event recording a spend.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventSpend {
    #[prost(message, optional, tag = "1")]
    pub nullifier: ::core::option::Option<super::super::sct::v1alpha1::Nullifier>,
}
impl ::prost::Name for EventSpend {
    const NAME: &'static str = "EventSpend";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// ABCI Event recording an output.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventOutput {
    #[prost(message, optional, tag = "1")]
    pub note_commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
}
impl ::prost::Name for EventOutput {
    const NAME: &'static str = "EventOutput";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// The body of a spend description, containing only the effecting data
/// describing changes to the ledger, and not the authorizing data that allows
/// those changes to be performed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendBody {
    /// A commitment to the value of the input note.
    #[prost(message, optional, tag = "1")]
    pub balance_commitment: ::core::option::Option<
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
    /// The nullifier of the input note.
    #[prost(message, optional, tag = "6")]
    pub nullifier: ::core::option::Option<super::super::sct::v1alpha1::Nullifier>,
    /// The randomized validating key for the spend authorization signature.
    #[prost(message, optional, tag = "4")]
    pub rk: ::core::option::Option<
        super::super::super::super::crypto::decaf377_rdsa::v1alpha1::SpendVerificationKey,
    >,
}
impl ::prost::Name for SpendBody {
    const NAME: &'static str = "SpendBody";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendView {
    #[prost(oneof = "spend_view::SpendView", tags = "1, 2")]
    pub spend_view: ::core::option::Option<spend_view::SpendView>,
}
/// Nested message and enum types in `SpendView`.
pub mod spend_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub spend: ::core::option::Option<super::Spend>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<super::NoteView>,
    }
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.shielded_pool.v1alpha1.SpendView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub spend: ::core::option::Option<super::Spend>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.shielded_pool.v1alpha1.SpendView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SpendView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
impl ::prost::Name for SpendView {
    const NAME: &'static str = "SpendView";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendPlan {
    /// The plaintext note we plan to spend.
    #[prost(message, optional, tag = "1")]
    pub note: ::core::option::Option<Note>,
    /// The position of the note we plan to spend.
    #[prost(uint64, tag = "2")]
    pub position: u64,
    /// The randomizer to use for the spend.
    #[prost(bytes = "vec", tag = "3")]
    pub randomizer: ::prost::alloc::vec::Vec<u8>,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes = "vec", tag = "4")]
    pub value_blinding: ::prost::alloc::vec::Vec<u8>,
    /// The first blinding factor to use for the ZK spend proof.
    #[prost(bytes = "vec", tag = "5")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK spend proof.
    #[prost(bytes = "vec", tag = "6")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SpendPlan {
    const NAME: &'static str = "SpendPlan";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// Creates a new shielded note.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Output {
    /// The effecting data for the output.
    #[prost(message, optional, tag = "1")]
    pub body: ::core::option::Option<OutputBody>,
    /// The output proof is authorizing data.
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<ZkOutputProof>,
}
impl ::prost::Name for Output {
    const NAME: &'static str = "Output";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// The body of an output description, containing only the effecting data
/// describing changes to the ledger, and not the authorizing data that allows
/// those changes to be performed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputBody {
    /// The minimal data required to scan and process the new output note.
    #[prost(message, optional, tag = "1")]
    pub note_payload: ::core::option::Option<NotePayload>,
    /// A commitment to the value of the output note. 32 bytes.
    #[prost(message, optional, tag = "2")]
    pub balance_commitment: ::core::option::Option<
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
    /// An encrypted key for decrypting the memo.
    #[prost(bytes = "vec", tag = "3")]
    pub wrapped_memo_key: ::prost::alloc::vec::Vec<u8>,
    /// The key material used for note encryption, wrapped in encryption to the
    /// sender's outgoing viewing key. 80 bytes.
    #[prost(bytes = "vec", tag = "4")]
    pub ovk_wrapped_key: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for OutputBody {
    const NAME: &'static str = "OutputBody";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputView {
    #[prost(oneof = "output_view::OutputView", tags = "1, 2")]
    pub output_view: ::core::option::Option<output_view::OutputView>,
}
/// Nested message and enum types in `OutputView`.
pub mod output_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub output: ::core::option::Option<super::Output>,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<super::NoteView>,
        #[prost(message, optional, tag = "3")]
        pub payload_key: ::core::option::Option<
            super::super::super::super::keys::v1alpha1::PayloadKey,
        >,
    }
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.shielded_pool.v1alpha1.OutputView.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub output: ::core::option::Option<super::Output>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.shielded_pool.v1alpha1.OutputView.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum OutputView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
impl ::prost::Name for OutputView {
    const NAME: &'static str = "OutputView";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutputPlan {
    /// The value to send to this output.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    /// The destination address to send it to.
    #[prost(message, optional, tag = "2")]
    pub dest_address: ::core::option::Option<
        super::super::super::keys::v1alpha1::Address,
    >,
    /// The rseed to use for the new note.
    #[prost(bytes = "vec", tag = "3")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
    /// The blinding factor to use for the value commitment.
    #[prost(bytes = "vec", tag = "4")]
    pub value_blinding: ::prost::alloc::vec::Vec<u8>,
    /// The first blinding factor to use for the ZK output proof.
    #[prost(bytes = "vec", tag = "5")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK output proof.
    #[prost(bytes = "vec", tag = "6")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for OutputPlan {
    const NAME: &'static str = "OutputPlan";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// Requests information on an asset by asset id
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomMetadataByIdRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The asset id to request information on.
    #[prost(message, optional, tag = "2")]
    pub asset_id: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
}
impl ::prost::Name for DenomMetadataByIdRequest {
    const NAME: &'static str = "DenomMetadataByIdRequest";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DenomMetadataByIdResponse {
    /// If present, information on the requested asset.
    ///
    /// If the requested asset was unknown, this field will not be present.
    #[prost(message, optional, tag = "1")]
    pub denom_metadata: ::core::option::Option<
        super::super::super::asset::v1alpha1::DenomMetadata,
    >,
}
impl ::prost::Name for DenomMetadataByIdResponse {
    const NAME: &'static str = "DenomMetadataByIdResponse";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// Genesis data for the shielded pool component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// The allocations present at genesis
    #[prost(message, repeated, tag = "2")]
    pub allocations: ::prost::alloc::vec::Vec<genesis_content::Allocation>,
}
/// Nested message and enum types in `GenesisContent`.
pub mod genesis_content {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Allocation {
        #[prost(message, optional, tag = "1")]
        pub amount: ::core::option::Option<
            super::super::super::super::num::v1alpha1::Amount,
        >,
        #[prost(string, tag = "2")]
        pub denom: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "3")]
        pub address: ::core::option::Option<
            super::super::super::super::keys::v1alpha1::Address,
        >,
    }
    impl ::prost::Name for Allocation {
        const NAME: &'static str = "Allocation";
        const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.shielded_pool.v1alpha1.GenesisContent.{}",
                Self::NAME
            )
        }
    }
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.shielded_pool.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.shielded_pool.v1alpha1.{}", Self::NAME
        )
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the shielded pool component.
    #[derive(Debug, Clone)]
    pub struct QueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn denom_metadata_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::DenomMetadataByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DenomMetadataByIdResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.shielded_pool.v1alpha1.QueryService/DenomMetadataById",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.shielded_pool.v1alpha1.QueryService",
                        "DenomMetadataById",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServiceServer.
    #[async_trait]
    pub trait QueryService: Send + Sync + 'static {
        async fn denom_metadata_by_id(
            &self,
            request: tonic::Request<super::DenomMetadataByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DenomMetadataByIdResponse>,
            tonic::Status,
        >;
    }
    /// Query operations for the shielded pool component.
    #[derive(Debug)]
    pub struct QueryServiceServer<T: QueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: QueryService> QueryServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServiceServer<T>
    where
        T: QueryService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.core.component.shielded_pool.v1alpha1.QueryService/DenomMetadataById" => {
                    #[allow(non_camel_case_types)]
                    struct DenomMetadataByIdSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::DenomMetadataByIdRequest>
                    for DenomMetadataByIdSvc<T> {
                        type Response = super::DenomMetadataByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DenomMetadataByIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::denom_metadata_by_id(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DenomMetadataByIdSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: QueryService> Clone for QueryServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: QueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: QueryService> tonic::server::NamedService for QueryServiceServer<T> {
        const NAME: &'static str = "penumbra.core.component.shielded_pool.v1alpha1.QueryService";
    }
}
