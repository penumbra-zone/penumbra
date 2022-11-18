/// Lists all assets in Asset Registry
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetListRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
}
/// Requests a range of compact block data.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CompactBlockRangeRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The start height of the range.
    #[prost(uint64, tag="2")]
    pub start_height: u64,
    /// The end height of the range.
    ///
    /// If unset, defaults to the latest block height.
    #[prost(uint64, tag="3")]
    pub end_height: u64,
    /// If set, keep the connection alive past end_height,
    /// streaming new compact blocks as they are created.
    #[prost(bool, tag="4")]
    pub keep_alive: bool,
}
/// Requests the governance-mutable parameters available for the chain.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MutableParametersRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
}
/// Requests the global configuration data for the chain.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainParamsRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
}
/// Requests information on the chain's validators.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorInfoRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
    /// Whether or not to return inactive validators
    #[prost(bool, tag="2")]
    pub show_inactive: bool,
}
/// Requests information on an asset by asset id
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetInfoRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The asset id to request information on.
    #[prost(message, optional, tag="2")]
    pub asset_id: ::core::option::Option<super::super::core::crypto::v1alpha1::AssetId>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AssetInfoResponse {
    /// If present, information on the requested asset.
    ///
    /// If the requested asset was unknown, this field will not be present.
    #[prost(message, optional, tag="1")]
    pub asset: ::core::option::Option<super::super::core::crypto::v1alpha1::Asset>,
}
/// Requests batch swap data associated with a given height and trading pair from the view service.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchSwapOutputDataRequest {
    #[prost(uint64, tag="1")]
    pub height: u64,
    #[prost(message, optional, tag="2")]
    pub trading_pair: ::core::option::Option<super::super::core::dex::v1alpha1::TradingPair>,
}
/// Requests CPMM reserves data associated with a given trading pair from the view service.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StubCpmmReservesRequest {
    #[prost(message, optional, tag="1")]
    pub trading_pair: ::core::option::Option<super::super::core::dex::v1alpha1::TradingPair>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorStatusRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub identity_key: ::core::option::Option<super::super::core::crypto::v1alpha1::IdentityKey>,
}
/// Performs a key-value query, either by key or by key hash.
///
/// Proofs are only supported by key.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValueRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag="1")]
    pub chain_id: ::prost::alloc::string::String,
    /// If set, the key to fetch from storage.
    #[prost(string, tag="2")]
    pub key: ::prost::alloc::string::String,
    /// whether to return a proof
    #[prost(bool, tag="3")]
    pub proof: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValueResponse {
    #[prost(bytes="vec", tag="1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="2")]
    pub proof: ::core::option::Option<::ibc_proto::ibc::core::commitment::v1::MerkleProof>,
}
/// Generated client implementations.
pub mod oblivious_query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Methods for accessing chain state that are "oblivious" in the sense that they
    /// do not request specific portions of the chain state that could reveal private
    /// client data.  For instance, requesting all asset denominations is oblivious,
    /// but requesting the asset denomination for a specific asset id is not, because
    /// it reveals that the client has an interest in that asset specifically.
    #[derive(Debug, Clone)]
    pub struct ObliviousQueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ObliviousQueryServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ObliviousQueryServiceClient<T>
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
        ) -> ObliviousQueryServiceClient<InterceptedService<T, F>>
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
            ObliviousQueryServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn compact_block_range(
            &mut self,
            request: impl tonic::IntoRequest<super::CompactBlockRangeRequest>,
        ) -> Result<
            tonic::Response<
                tonic::codec::Streaming<
                    super::super::super::core::chain::v1alpha1::CompactBlock,
                >,
            >,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/CompactBlockRange",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn chain_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::ChainParamsRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::chain::v1alpha1::ChainParameters>,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn mutable_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::MutableParametersRequest>,
        ) -> Result<
            tonic::Response<
                tonic::codec::Streaming<
                    super::super::super::core::governance::v1alpha1::MutableChainParameter,
                >,
            >,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/MutableParameters",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn validator_info(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorInfoRequest>,
        ) -> Result<
            tonic::Response<
                tonic::codec::Streaming<
                    super::super::super::core::stake::v1alpha1::ValidatorInfo,
                >,
            >,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/ValidatorInfo",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// TODO: deprecate in favor of SpecificQuery.AssetInfo
        pub async fn asset_list(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetListRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::chain::v1alpha1::KnownAssets>,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/AssetList",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod specific_query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Methods for accessing chain state that are "specific" in the sense that they
    /// request specific portions of the chain state that could reveal private
    /// client data.  For instance, requesting all asset denominations is oblivious,
    /// but requesting the asset denomination for a specific asset id is not, because
    /// it reveals that the client has an interest in that asset specifically.
    #[derive(Debug, Clone)]
    pub struct SpecificQueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SpecificQueryServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> SpecificQueryServiceClient<T>
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
        ) -> SpecificQueryServiceClient<InterceptedService<T, F>>
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
            SpecificQueryServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn transaction_by_note(
            &mut self,
            request: impl tonic::IntoRequest<
                super::super::super::core::crypto::v1alpha1::NoteCommitment,
            >,
        ) -> Result<
            tonic::Response<super::super::super::core::chain::v1alpha1::NoteSource>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/TransactionByNote",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn validator_status(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorStatusRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::stake::v1alpha1::ValidatorStatus>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ValidatorStatus",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn next_validator_rate(
            &mut self,
            request: impl tonic::IntoRequest<
                super::super::super::core::crypto::v1alpha1::IdentityKey,
            >,
        ) -> Result<
            tonic::Response<super::super::super::core::stake::v1alpha1::RateData>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/NextValidatorRate",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn batch_swap_output_data(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchSwapOutputDataRequest>,
        ) -> Result<
            tonic::Response<
                super::super::super::core::dex::v1alpha1::BatchSwapOutputData,
            >,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/BatchSwapOutputData",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn stub_cpmm_reserves(
            &mut self,
            request: impl tonic::IntoRequest<super::StubCpmmReservesRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::dex::v1alpha1::Reserves>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/StubCPMMReserves",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn asset_info(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetInfoRequest>,
        ) -> Result<tonic::Response<super::AssetInfoResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/AssetInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// General-purpose key-value state query API, that can be used to query
        /// arbitrary keys in the JMT storage.
        pub async fn key_value(
            &mut self,
            request: impl tonic::IntoRequest<super::KeyValueRequest>,
        ) -> Result<tonic::Response<super::KeyValueResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/KeyValue",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod oblivious_query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with ObliviousQueryServiceServer.
    #[async_trait]
    pub trait ObliviousQueryService: Send + Sync + 'static {
        ///Server streaming response type for the CompactBlockRange method.
        type CompactBlockRangeStream: futures_core::Stream<
                Item = Result<
                    super::super::super::core::chain::v1alpha1::CompactBlock,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        async fn compact_block_range(
            &self,
            request: tonic::Request<super::CompactBlockRangeRequest>,
        ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, tonic::Status>;
        async fn chain_parameters(
            &self,
            request: tonic::Request<super::ChainParamsRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::chain::v1alpha1::ChainParameters>,
            tonic::Status,
        >;
        ///Server streaming response type for the MutableParameters method.
        type MutableParametersStream: futures_core::Stream<
                Item = Result<
                    super::super::super::core::governance::v1alpha1::MutableChainParameter,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        async fn mutable_parameters(
            &self,
            request: tonic::Request<super::MutableParametersRequest>,
        ) -> Result<tonic::Response<Self::MutableParametersStream>, tonic::Status>;
        ///Server streaming response type for the ValidatorInfo method.
        type ValidatorInfoStream: futures_core::Stream<
                Item = Result<
                    super::super::super::core::stake::v1alpha1::ValidatorInfo,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        async fn validator_info(
            &self,
            request: tonic::Request<super::ValidatorInfoRequest>,
        ) -> Result<tonic::Response<Self::ValidatorInfoStream>, tonic::Status>;
        /// TODO: deprecate in favor of SpecificQuery.AssetInfo
        async fn asset_list(
            &self,
            request: tonic::Request<super::AssetListRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::chain::v1alpha1::KnownAssets>,
            tonic::Status,
        >;
    }
    /// Methods for accessing chain state that are "oblivious" in the sense that they
    /// do not request specific portions of the chain state that could reveal private
    /// client data.  For instance, requesting all asset denominations is oblivious,
    /// but requesting the asset denomination for a specific asset id is not, because
    /// it reveals that the client has an interest in that asset specifically.
    #[derive(Debug)]
    pub struct ObliviousQueryServiceServer<T: ObliviousQueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ObliviousQueryService> ObliviousQueryServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
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
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for ObliviousQueryServiceServer<T>
    where
        T: ObliviousQueryService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.client.v1alpha1.ObliviousQueryService/CompactBlockRange" => {
                    #[allow(non_camel_case_types)]
                    struct CompactBlockRangeSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::ServerStreamingService<
                        super::CompactBlockRangeRequest,
                    > for CompactBlockRangeSvc<T> {
                        type Response = super::super::super::core::chain::v1alpha1::CompactBlock;
                        type ResponseStream = T::CompactBlockRangeStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CompactBlockRangeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).compact_block_range(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CompactBlockRangeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters" => {
                    #[allow(non_camel_case_types)]
                    struct ChainParametersSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::UnaryService<super::ChainParamsRequest>
                    for ChainParametersSvc<T> {
                        type Response = super::super::super::core::chain::v1alpha1::ChainParameters;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ChainParamsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).chain_parameters(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChainParametersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.ObliviousQueryService/MutableParameters" => {
                    #[allow(non_camel_case_types)]
                    struct MutableParametersSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::ServerStreamingService<
                        super::MutableParametersRequest,
                    > for MutableParametersSvc<T> {
                        type Response = super::super::super::core::governance::v1alpha1::MutableChainParameter;
                        type ResponseStream = T::MutableParametersStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MutableParametersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).mutable_parameters(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = MutableParametersSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.ObliviousQueryService/ValidatorInfo" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorInfoSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::ServerStreamingService<super::ValidatorInfoRequest>
                    for ValidatorInfoSvc<T> {
                        type Response = super::super::super::core::stake::v1alpha1::ValidatorInfo;
                        type ResponseStream = T::ValidatorInfoStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).validator_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidatorInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.ObliviousQueryService/AssetList" => {
                    #[allow(non_camel_case_types)]
                    struct AssetListSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::UnaryService<super::AssetListRequest>
                    for AssetListSvc<T> {
                        type Response = super::super::super::core::chain::v1alpha1::KnownAssets;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AssetListRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).asset_list(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AssetListSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
    impl<T: ObliviousQueryService> Clone for ObliviousQueryServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ObliviousQueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ObliviousQueryService> tonic::server::NamedService
    for ObliviousQueryServiceServer<T> {
        const NAME: &'static str = "penumbra.client.v1alpha1.ObliviousQueryService";
    }
}
/// Generated server implementations.
pub mod specific_query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with SpecificQueryServiceServer.
    #[async_trait]
    pub trait SpecificQueryService: Send + Sync + 'static {
        async fn transaction_by_note(
            &self,
            request: tonic::Request<
                super::super::super::core::crypto::v1alpha1::NoteCommitment,
            >,
        ) -> Result<
            tonic::Response<super::super::super::core::chain::v1alpha1::NoteSource>,
            tonic::Status,
        >;
        async fn validator_status(
            &self,
            request: tonic::Request<super::ValidatorStatusRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::stake::v1alpha1::ValidatorStatus>,
            tonic::Status,
        >;
        async fn next_validator_rate(
            &self,
            request: tonic::Request<
                super::super::super::core::crypto::v1alpha1::IdentityKey,
            >,
        ) -> Result<
            tonic::Response<super::super::super::core::stake::v1alpha1::RateData>,
            tonic::Status,
        >;
        async fn batch_swap_output_data(
            &self,
            request: tonic::Request<super::BatchSwapOutputDataRequest>,
        ) -> Result<
            tonic::Response<
                super::super::super::core::dex::v1alpha1::BatchSwapOutputData,
            >,
            tonic::Status,
        >;
        async fn stub_cpmm_reserves(
            &self,
            request: tonic::Request<super::StubCpmmReservesRequest>,
        ) -> Result<
            tonic::Response<super::super::super::core::dex::v1alpha1::Reserves>,
            tonic::Status,
        >;
        async fn asset_info(
            &self,
            request: tonic::Request<super::AssetInfoRequest>,
        ) -> Result<tonic::Response<super::AssetInfoResponse>, tonic::Status>;
        /// General-purpose key-value state query API, that can be used to query
        /// arbitrary keys in the JMT storage.
        async fn key_value(
            &self,
            request: tonic::Request<super::KeyValueRequest>,
        ) -> Result<tonic::Response<super::KeyValueResponse>, tonic::Status>;
    }
    /// Methods for accessing chain state that are "specific" in the sense that they
    /// request specific portions of the chain state that could reveal private
    /// client data.  For instance, requesting all asset denominations is oblivious,
    /// but requesting the asset denomination for a specific asset id is not, because
    /// it reveals that the client has an interest in that asset specifically.
    #[derive(Debug)]
    pub struct SpecificQueryServiceServer<T: SpecificQueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: SpecificQueryService> SpecificQueryServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
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
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for SpecificQueryServiceServer<T>
    where
        T: SpecificQueryService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.client.v1alpha1.SpecificQueryService/TransactionByNote" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionByNoteSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<
                        super::super::super::core::crypto::v1alpha1::NoteCommitment,
                    > for TransactionByNoteSvc<T> {
                        type Response = super::super::super::core::chain::v1alpha1::NoteSource;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::core::crypto::v1alpha1::NoteCommitment,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_by_note(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionByNoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.SpecificQueryService/ValidatorStatus" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorStatusSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::ValidatorStatusRequest>
                    for ValidatorStatusSvc<T> {
                        type Response = super::super::super::core::stake::v1alpha1::ValidatorStatus;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).validator_status(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidatorStatusSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.SpecificQueryService/NextValidatorRate" => {
                    #[allow(non_camel_case_types)]
                    struct NextValidatorRateSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<
                        super::super::super::core::crypto::v1alpha1::IdentityKey,
                    > for NextValidatorRateSvc<T> {
                        type Response = super::super::super::core::stake::v1alpha1::RateData;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::super::core::crypto::v1alpha1::IdentityKey,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).next_validator_rate(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NextValidatorRateSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.SpecificQueryService/BatchSwapOutputData" => {
                    #[allow(non_camel_case_types)]
                    struct BatchSwapOutputDataSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::BatchSwapOutputDataRequest>
                    for BatchSwapOutputDataSvc<T> {
                        type Response = super::super::super::core::dex::v1alpha1::BatchSwapOutputData;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchSwapOutputDataRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).batch_swap_output_data(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BatchSwapOutputDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.SpecificQueryService/StubCPMMReserves" => {
                    #[allow(non_camel_case_types)]
                    struct StubCPMMReservesSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::StubCpmmReservesRequest>
                    for StubCPMMReservesSvc<T> {
                        type Response = super::super::super::core::dex::v1alpha1::Reserves;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StubCpmmReservesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).stub_cpmm_reserves(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StubCPMMReservesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.SpecificQueryService/AssetInfo" => {
                    #[allow(non_camel_case_types)]
                    struct AssetInfoSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::AssetInfoRequest>
                    for AssetInfoSvc<T> {
                        type Response = super::AssetInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AssetInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).asset_info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AssetInfoSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.client.v1alpha1.SpecificQueryService/KeyValue" => {
                    #[allow(non_camel_case_types)]
                    struct KeyValueSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::KeyValueRequest>
                    for KeyValueSvc<T> {
                        type Response = super::KeyValueResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::KeyValueRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).key_value(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = KeyValueSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
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
    impl<T: SpecificQueryService> Clone for SpecificQueryServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: SpecificQueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SpecificQueryService> tonic::server::NamedService
    for SpecificQueryServiceServer<T> {
        const NAME: &'static str = "penumbra.client.v1alpha1.SpecificQueryService";
    }
}
