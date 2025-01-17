/// Configuration data for the SCT component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SctParameters {
    /// The default duration of each epoch, in number of blocks.
    #[prost(uint64, tag = "1")]
    pub epoch_duration: u64,
}
impl ::prost::Name for SctParameters {
    const NAME: &'static str = "SctParameters";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Sct-specific genesis content.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// The SctParameters present at genesis.
    #[prost(message, optional, tag = "1")]
    pub sct_params: ::core::option::Option<SctParameters>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// An epoch is a sequentially numbered collection of contiguous blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Epoch {
    /// The unique index of the epoch.
    #[prost(uint64, tag = "1")]
    pub index: u64,
    /// The starting height for the epoch.
    #[prost(uint64, tag = "2")]
    pub start_height: u64,
}
impl ::prost::Name for Epoch {
    const NAME: &'static str = "Epoch";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Metadata describing the source of a commitment in the state commitment tree.
///
/// This message allows clients to track provenance of state commitments, and to
/// decide whether or not to download block data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitmentSource {
    #[prost(oneof = "commitment_source::Source", tags = "1, 2, 20, 30, 40")]
    pub source: ::core::option::Option<commitment_source::Source>,
}
/// Nested message and enum types in `CommitmentSource`.
pub mod commitment_source {
    /// The state commitment was included in the genesis state.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Genesis {}
    impl ::prost::Name for Genesis {
        const NAME: &'static str = "Genesis";
        const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.sct.v1.CommitmentSource.{}", Self::NAME
            )
        }
    }
    /// The commitment was created by a transaction.
    ///
    /// When included in a `CompactBlock` via a `StatePayload`, the transaction source is "dehydrated"
    /// by stripping the `id` field and putting empty bytes in its place.  When clients perform extended
    /// transaction fetch, they should match up transaction hashes to "rehydrate" the source info.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Transaction {
        /// The transaction ID, if specified.
        ///
        /// This field may be omitted to save space, and should not be required to be present.
        /// If the bytes are missing, the message should be interpreted as "Transaction (Unknown)".
        #[prost(bytes = "vec", tag = "1")]
        pub id: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for Transaction {
        const NAME: &'static str = "Transaction";
        const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.sct.v1.CommitmentSource.{}", Self::NAME
            )
        }
    }
    /// The commitment was created through a validator's funding stream.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FundingStreamReward {
        /// The epoch index the rewards were issued in.
        #[prost(uint64, tag = "1")]
        pub epoch_index: u64,
    }
    impl ::prost::Name for FundingStreamReward {
        const NAME: &'static str = "FundingStreamReward";
        const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.sct.v1.CommitmentSource.{}", Self::NAME
            )
        }
    }
    /// The commitment was created through a `CommunityPoolOutput` in a governance-initated transaction.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CommunityPoolOutput {}
    impl ::prost::Name for CommunityPoolOutput {
        const NAME: &'static str = "CommunityPoolOutput";
        const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.sct.v1.CommitmentSource.{}", Self::NAME
            )
        }
    }
    /// The commitment was created by an inbound ICS20 transfer.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Ics20Transfer {
        /// The sequence number of the packet that triggered the transfer
        #[prost(uint64, tag = "1")]
        pub packet_seq: u64,
        /// The channel id the transfer happened on
        #[prost(string, tag = "2")]
        pub channel_id: ::prost::alloc::string::String,
        /// The sender address on the counterparty chain
        #[prost(string, tag = "3")]
        pub sender: ::prost::alloc::string::String,
    }
    impl ::prost::Name for Ics20Transfer {
        const NAME: &'static str = "Ics20Transfer";
        const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.sct.v1.CommitmentSource.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Source {
        #[prost(message, tag = "1")]
        Transaction(Transaction),
        #[prost(message, tag = "2")]
        Ics20Transfer(Ics20Transfer),
        #[prost(message, tag = "20")]
        FundingStreamReward(FundingStreamReward),
        #[prost(message, tag = "30")]
        CommunityPoolOutput(CommunityPoolOutput),
        #[prost(message, tag = "40")]
        Genesis(Genesis),
    }
}
impl ::prost::Name for CommitmentSource {
    const NAME: &'static str = "CommitmentSource";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Nullifier {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Nullifier {
    const NAME: &'static str = "Nullifier";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Records information about what transaction spent a nullifier.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NullificationInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub spend_height: u64,
}
impl ::prost::Name for NullificationInfo {
    const NAME: &'static str = "NullificationInfo";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Event recording a new commitment added to the SCT.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCommitment {
    #[prost(message, optional, tag = "1")]
    pub commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1::StateCommitment,
    >,
    #[prost(uint64, tag = "2")]
    pub position: u64,
    #[prost(message, optional, tag = "3")]
    pub source: ::core::option::Option<CommitmentSource>,
}
impl ::prost::Name for EventCommitment {
    const NAME: &'static str = "EventCommitment";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Event recording an SCT anchor (global root).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAnchor {
    #[prost(message, optional, tag = "1")]
    pub anchor: ::core::option::Option<
        super::super::super::super::crypto::tct::v1::MerkleRoot,
    >,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for EventAnchor {
    const NAME: &'static str = "EventAnchor";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Event recording an SCT epoch root.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventEpochRoot {
    #[prost(message, optional, tag = "1")]
    pub root: ::core::option::Option<
        super::super::super::super::crypto::tct::v1::MerkleRoot,
    >,
    #[prost(uint64, tag = "2")]
    pub index: u64,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for EventEpochRoot {
    const NAME: &'static str = "EventEpochRoot";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Event recording an SCT block root.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventBlockRoot {
    #[prost(message, optional, tag = "1")]
    pub root: ::core::option::Option<
        super::super::super::super::crypto::tct::v1::MerkleRoot,
    >,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for EventBlockRoot {
    const NAME: &'static str = "EventBlockRoot";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochByHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for EpochByHeightRequest {
    const NAME: &'static str = "EpochByHeightRequest";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub epoch: ::core::option::Option<Epoch>,
}
impl ::prost::Name for EpochByHeightResponse {
    const NAME: &'static str = "EpochByHeightResponse";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnchorByHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for AnchorByHeightRequest {
    const NAME: &'static str = "AnchorByHeightRequest";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnchorByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub anchor: ::core::option::Option<
        super::super::super::super::crypto::tct::v1::MerkleRoot,
    >,
}
impl ::prost::Name for AnchorByHeightResponse {
    const NAME: &'static str = "AnchorByHeightResponse";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimestampByHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
impl ::prost::Name for TimestampByHeightRequest {
    const NAME: &'static str = "TimestampByHeightRequest";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimestampByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for TimestampByHeightResponse {
    const NAME: &'static str = "TimestampByHeightResponse";
    const PACKAGE: &'static str = "penumbra.core.component.sct.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.sct.v1.{}", Self::NAME)
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the SCT component.
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
        pub async fn anchor_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::AnchorByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AnchorByHeightResponse>,
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
                "/penumbra.core.component.sct.v1.QueryService/AnchorByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.sct.v1.QueryService",
                        "AnchorByHeight",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn epoch_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::EpochByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EpochByHeightResponse>,
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
                "/penumbra.core.component.sct.v1.QueryService/EpochByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.sct.v1.QueryService",
                        "EpochByHeight",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn timestamp_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::TimestampByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TimestampByHeightResponse>,
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
                "/penumbra.core.component.sct.v1.QueryService/TimestampByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.sct.v1.QueryService",
                        "TimestampByHeight",
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
        async fn anchor_by_height(
            &self,
            request: tonic::Request<super::AnchorByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AnchorByHeightResponse>,
            tonic::Status,
        >;
        async fn epoch_by_height(
            &self,
            request: tonic::Request<super::EpochByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EpochByHeightResponse>,
            tonic::Status,
        >;
        async fn timestamp_by_height(
            &self,
            request: tonic::Request<super::TimestampByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TimestampByHeightResponse>,
            tonic::Status,
        >;
    }
    /// Query operations for the SCT component.
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
                "/penumbra.core.component.sct.v1.QueryService/AnchorByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct AnchorByHeightSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::AnchorByHeightRequest>
                    for AnchorByHeightSvc<T> {
                        type Response = super::AnchorByHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AnchorByHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::anchor_by_height(&inner, request).await
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
                        let method = AnchorByHeightSvc(inner);
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
                "/penumbra.core.component.sct.v1.QueryService/EpochByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct EpochByHeightSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::EpochByHeightRequest>
                    for EpochByHeightSvc<T> {
                        type Response = super::EpochByHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EpochByHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::epoch_by_height(&inner, request).await
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
                        let method = EpochByHeightSvc(inner);
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
                "/penumbra.core.component.sct.v1.QueryService/TimestampByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct TimestampByHeightSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::TimestampByHeightRequest>
                    for TimestampByHeightSvc<T> {
                        type Response = super::TimestampByHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TimestampByHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::timestamp_by_height(&inner, request)
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
                        let method = TimestampByHeightSvc(inner);
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
        const NAME: &'static str = "penumbra.core.component.sct.v1.QueryService";
    }
}
