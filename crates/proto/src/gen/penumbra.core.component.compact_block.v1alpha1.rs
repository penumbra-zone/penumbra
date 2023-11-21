/// Contains the minimum data needed to update client state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBlock {
    #[prost(uint64, tag = "1")]
    pub height: u64,
    /// State payloads describing new state fragments.
    #[prost(message, repeated, tag = "2")]
    pub state_payloads: ::prost::alloc::vec::Vec<StatePayload>,
    /// Nullifiers identifying spent notes.
    #[prost(message, repeated, tag = "3")]
    pub nullifiers: ::prost::alloc::vec::Vec<super::super::sct::v1alpha1::Nullifier>,
    /// The block root of this block.
    #[prost(message, optional, tag = "4")]
    pub block_root: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
    /// The epoch root of this epoch (only present when the block is the last in an epoch).
    #[prost(message, optional, tag = "17")]
    pub epoch_root: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::MerkleRoot,
    >,
    /// If a proposal started voting in this block, this is set to `true`.
    #[prost(bool, tag = "20")]
    pub proposal_started: bool,
    /// Latest Fuzzy Message Detection parameters.
    #[prost(message, optional, tag = "100")]
    pub fmd_parameters: ::core::option::Option<
        super::super::chain::v1alpha1::FmdParameters,
    >,
    /// Price data for swaps executed in this block.
    #[prost(message, repeated, tag = "5")]
    pub swap_outputs: ::prost::alloc::vec::Vec<
        super::super::dex::v1alpha1::BatchSwapOutputData,
    >,
    /// Indicates updated app parameters.
    #[prost(bool, tag = "6")]
    pub app_parameters_updated: bool,
    /// Updated gas prices, if they have changed.
    #[prost(message, optional, tag = "7")]
    pub gas_prices: ::core::option::Option<super::super::fee::v1alpha1::GasPrices>,
}
impl ::prost::Name for CompactBlock {
    const NAME: &'static str = "CompactBlock";
    const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.compact_block.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatePayload {
    #[prost(oneof = "state_payload::StatePayload", tags = "1, 2, 3")]
    pub state_payload: ::core::option::Option<state_payload::StatePayload>,
}
/// Nested message and enum types in `StatePayload`.
pub mod state_payload {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RolledUp {
        #[prost(message, optional, tag = "1")]
        pub commitment: ::core::option::Option<
            super::super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
        >,
    }
    impl ::prost::Name for RolledUp {
        const NAME: &'static str = "RolledUp";
        const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.compact_block.v1alpha1.StatePayload.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Note {
        #[prost(message, optional, tag = "1")]
        pub source: ::core::option::Option<
            super::super::super::chain::v1alpha1::NoteSource,
        >,
        #[prost(message, optional, tag = "2")]
        pub note: ::core::option::Option<
            super::super::super::shielded_pool::v1alpha1::NotePayload,
        >,
    }
    impl ::prost::Name for Note {
        const NAME: &'static str = "Note";
        const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.compact_block.v1alpha1.StatePayload.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Swap {
        #[prost(message, optional, tag = "1")]
        pub source: ::core::option::Option<
            super::super::super::chain::v1alpha1::NoteSource,
        >,
        #[prost(message, optional, tag = "2")]
        pub swap: ::core::option::Option<
            super::super::super::dex::v1alpha1::SwapPayload,
        >,
    }
    impl ::prost::Name for Swap {
        const NAME: &'static str = "Swap";
        const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.compact_block.v1alpha1.StatePayload.{}",
                Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum StatePayload {
        #[prost(message, tag = "1")]
        RolledUp(RolledUp),
        #[prost(message, tag = "2")]
        Note(Note),
        #[prost(message, tag = "3")]
        Swap(Swap),
    }
}
impl ::prost::Name for StatePayload {
    const NAME: &'static str = "StatePayload";
    const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.compact_block.v1alpha1.{}", Self::NAME
        )
    }
}
/// Requests a range of compact block data.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBlockRangeRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The start height of the range.
    #[prost(uint64, tag = "2")]
    pub start_height: u64,
    /// The end height of the range, defaults to the latest block height.
    #[prost(uint64, tag = "3")]
    pub end_height: u64,
    /// If set, keeps the connection alive past `end_height`,
    /// streaming new compact blocks as they are created.
    #[prost(bool, tag = "4")]
    pub keep_alive: bool,
}
impl ::prost::Name for CompactBlockRangeRequest {
    const NAME: &'static str = "CompactBlockRangeRequest";
    const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.compact_block.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBlockRangeResponse {
    #[prost(message, optional, tag = "1")]
    pub compact_block: ::core::option::Option<CompactBlock>,
}
impl ::prost::Name for CompactBlockRangeResponse {
    const NAME: &'static str = "CompactBlockRangeResponse";
    const PACKAGE: &'static str = "penumbra.core.component.compact_block.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.compact_block.v1alpha1.{}", Self::NAME
        )
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the compact block component.
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
        /// Returns a stream of `CompactBlockRangeResponse`s.
        pub async fn compact_block_range(
            &mut self,
            request: impl tonic::IntoRequest<super::CompactBlockRangeRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::CompactBlockRangeResponse>>,
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
                "/penumbra.core.component.compact_block.v1alpha1.QueryService/CompactBlockRange",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.compact_block.v1alpha1.QueryService",
                        "CompactBlockRange",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
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
        /// Server streaming response type for the CompactBlockRange method.
        type CompactBlockRangeStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::CompactBlockRangeResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        /// Returns a stream of `CompactBlockRangeResponse`s.
        async fn compact_block_range(
            &self,
            request: tonic::Request<super::CompactBlockRangeRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::CompactBlockRangeStream>,
            tonic::Status,
        >;
    }
    /// Query operations for the compact block component.
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
                "/penumbra.core.component.compact_block.v1alpha1.QueryService/CompactBlockRange" => {
                    #[allow(non_camel_case_types)]
                    struct CompactBlockRangeSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<
                        super::CompactBlockRangeRequest,
                    > for CompactBlockRangeSvc<T> {
                        type Response = super::CompactBlockRangeResponse;
                        type ResponseStream = T::CompactBlockRangeStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CompactBlockRangeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::compact_block_range(&inner, request)
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
                        let method = CompactBlockRangeSvc(inner);
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
                        let res = grpc.server_streaming(method, req).await;
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
        const NAME: &'static str = "penumbra.core.component.compact_block.v1alpha1.QueryService";
    }
}
