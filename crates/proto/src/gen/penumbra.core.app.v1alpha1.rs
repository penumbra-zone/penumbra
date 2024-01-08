/// Requests the list of all transactions that occurred within a given block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionsByHeightRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The block height to retrieve.
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
}
impl ::prost::Name for TransactionsByHeightRequest {
    const NAME: &'static str = "TransactionsByHeightRequest";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction that appeared within a given block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionsByHeightResponse {
    /// The transactions.
    #[prost(message, repeated, tag = "1")]
    pub transactions: ::prost::alloc::vec::Vec<
        super::super::transaction::v1alpha1::Transaction,
    >,
    /// The block height.
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
}
impl ::prost::Name for TransactionsByHeightResponse {
    const NAME: &'static str = "TransactionsByHeightResponse";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppParameters {
    /// Chain module parameters.
    #[prost(message, optional, tag = "1")]
    pub chain_params: ::core::option::Option<
        super::super::component::chain::v1alpha1::ChainParameters,
    >,
    /// Community Pool module parameters.
    #[prost(message, optional, tag = "2")]
    pub community_pool_params: ::core::option::Option<
        super::super::component::community_pool::v1alpha1::CommunityPoolParameters,
    >,
    /// Governance module parameters.
    #[prost(message, optional, tag = "3")]
    pub governance_params: ::core::option::Option<
        super::super::component::governance::v1alpha1::GovernanceParameters,
    >,
    /// IBC module parameters.
    #[prost(message, optional, tag = "4")]
    pub ibc_params: ::core::option::Option<
        super::super::component::ibc::v1alpha1::IbcParameters,
    >,
    /// Stake module parameters.
    #[prost(message, optional, tag = "5")]
    pub stake_params: ::core::option::Option<
        super::super::component::stake::v1alpha1::StakeParameters,
    >,
    /// Fee module parameters.
    #[prost(message, optional, tag = "6")]
    pub fee_params: ::core::option::Option<
        super::super::component::fee::v1alpha1::FeeParameters,
    >,
    /// Distributions module parameters.
    #[prost(message, optional, tag = "7")]
    pub distributions_params: ::core::option::Option<
        super::super::component::distributions::v1alpha1::DistributionsParameters,
    >,
}
impl ::prost::Name for AppParameters {
    const NAME: &'static str = "AppParameters";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
/// Requests the global configuration data for the app.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppParametersRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for AppParametersRequest {
    const NAME: &'static str = "AppParametersRequest";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AppParametersResponse {
    #[prost(message, optional, tag = "1")]
    pub app_parameters: ::core::option::Option<AppParameters>,
}
impl ::prost::Name for AppParametersResponse {
    const NAME: &'static str = "AppParametersResponse";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisAppState {
    #[prost(oneof = "genesis_app_state::GenesisAppState", tags = "1, 2")]
    pub genesis_app_state: ::core::option::Option<genesis_app_state::GenesisAppState>,
}
/// Nested message and enum types in `GenesisAppState`.
pub mod genesis_app_state {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum GenesisAppState {
        #[prost(message, tag = "1")]
        GenesisContent(super::GenesisContent),
        #[prost(bytes, tag = "2")]
        GenesisCheckpoint(::prost::alloc::vec::Vec<u8>),
    }
}
impl ::prost::Name for GenesisAppState {
    const NAME: &'static str = "GenesisAppState";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// Stake module genesis state.
    #[prost(message, optional, tag = "1")]
    pub stake_content: ::core::option::Option<
        super::super::component::stake::v1alpha1::GenesisContent,
    >,
    /// Shielded pool module genesis state.
    #[prost(message, optional, tag = "2")]
    pub shielded_pool_content: ::core::option::Option<
        super::super::component::shielded_pool::v1alpha1::GenesisContent,
    >,
    /// Governance module genesis state.
    #[prost(message, optional, tag = "3")]
    pub governance_content: ::core::option::Option<
        super::super::component::governance::v1alpha1::GenesisContent,
    >,
    /// IBC module genesis state.
    #[prost(message, optional, tag = "4")]
    pub ibc_content: ::core::option::Option<
        super::super::component::ibc::v1alpha1::GenesisContent,
    >,
    /// Chain module genesis state.
    #[prost(message, optional, tag = "5")]
    pub chain_content: ::core::option::Option<
        super::super::component::chain::v1alpha1::GenesisContent,
    >,
    /// Community Pool module genesis state.
    #[prost(message, optional, tag = "6")]
    pub community_pool_content: ::core::option::Option<
        super::super::component::community_pool::v1alpha1::GenesisContent,
    >,
    /// Fee module genesis state.
    #[prost(message, optional, tag = "7")]
    pub fee_content: ::core::option::Option<
        super::super::component::fee::v1alpha1::GenesisContent,
    >,
    /// Distributions module genesis state.
    #[prost(message, optional, tag = "8")]
    pub distributions_content: ::core::option::Option<
        super::super::component::distributions::v1alpha1::GenesisContent,
    >,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the overall Penumbra application.
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
        /// Gets the app parameters.
        pub async fn app_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::AppParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AppParametersResponse>,
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
                "/penumbra.core.app.v1alpha1.QueryService/AppParameters",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.app.v1alpha1.QueryService",
                        "AppParameters",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Returns the CometBFT transactions that occurred during a given block.
        pub async fn transactions_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionsByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsByHeightResponse>,
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
                "/penumbra.core.app.v1alpha1.QueryService/TransactionsByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.app.v1alpha1.QueryService",
                        "TransactionsByHeight",
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
        /// Gets the app parameters.
        async fn app_parameters(
            &self,
            request: tonic::Request<super::AppParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AppParametersResponse>,
            tonic::Status,
        >;
        /// Returns the CometBFT transactions that occurred during a given block.
        async fn transactions_by_height(
            &self,
            request: tonic::Request<super::TransactionsByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsByHeightResponse>,
            tonic::Status,
        >;
    }
    /// Query operations for the overall Penumbra application.
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
                "/penumbra.core.app.v1alpha1.QueryService/AppParameters" => {
                    #[allow(non_camel_case_types)]
                    struct AppParametersSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::AppParametersRequest>
                    for AppParametersSvc<T> {
                        type Response = super::AppParametersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AppParametersRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::app_parameters(&inner, request).await
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
                        let method = AppParametersSvc(inner);
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
                "/penumbra.core.app.v1alpha1.QueryService/TransactionsByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionsByHeightSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::TransactionsByHeightRequest>
                    for TransactionsByHeightSvc<T> {
                        type Response = super::TransactionsByHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionsByHeightRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::transactions_by_height(&inner, request)
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
                        let method = TransactionsByHeightSvc(inner);
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
        const NAME: &'static str = "penumbra.core.app.v1alpha1.QueryService";
    }
}
