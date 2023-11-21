/// Performs a key-value query, either by key or by key hash.
///
/// Proofs are only supported by key.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValueRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// If set, the key to fetch from storage.
    #[prost(string, tag = "2")]
    pub key: ::prost::alloc::string::String,
    /// whether to return a proof
    #[prost(bool, tag = "3")]
    pub proof: bool,
}
impl ::prost::Name for KeyValueRequest {
    const NAME: &'static str = "KeyValueRequest";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KeyValueResponse {
    /// The value corresponding to the specified key, if it was found.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<key_value_response::Value>,
    /// A proof of existence or non-existence.
    #[prost(message, optional, tag = "2")]
    pub proof: ::core::option::Option<
        ::ibc_proto::ibc::core::commitment::v1::MerkleProof,
    >,
}
/// Nested message and enum types in `KeyValueResponse`.
pub mod key_value_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Value {
        #[prost(bytes = "vec", tag = "1")]
        pub value: ::prost::alloc::vec::Vec<u8>,
    }
    impl ::prost::Name for Value {
        const NAME: &'static str = "Value";
        const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.app.v1alpha1.KeyValueResponse.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for KeyValueResponse {
    const NAME: &'static str = "KeyValueResponse";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
/// Performs a prefixed key-value query, by string prefix.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrefixValueRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The prefix to fetch subkeys from storage.
    #[prost(string, tag = "2")]
    pub prefix: ::prost::alloc::string::String,
}
impl ::prost::Name for PrefixValueRequest {
    const NAME: &'static str = "PrefixValueRequest";
    const PACKAGE: &'static str = "penumbra.core.app.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.app.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrefixValueResponse {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PrefixValueResponse {
    const NAME: &'static str = "PrefixValueResponse";
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
    /// DAO module parameters.
    #[prost(message, optional, tag = "2")]
    pub dao_params: ::core::option::Option<
        super::super::component::dao::v1alpha1::DaoParameters,
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
    /// DAO module genesis state.
    #[prost(message, optional, tag = "6")]
    pub dao_content: ::core::option::Option<
        super::super::component::dao::v1alpha1::GenesisContent,
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
        /// General-purpose key-value state query API, that can be used to query
        /// arbitrary keys in the JMT storage.
        pub async fn key_value(
            &mut self,
            request: impl tonic::IntoRequest<super::KeyValueRequest>,
        ) -> std::result::Result<
            tonic::Response<super::KeyValueResponse>,
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
                "/penumbra.core.app.v1alpha1.QueryService/KeyValue",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.app.v1alpha1.QueryService",
                        "KeyValue",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// General-purpose prefixed key-value state query API, that can be used to query
        /// arbitrary prefixes in the JMT storage.
        /// Returns a stream of `PrefixValueResponse`s.
        pub async fn prefix_value(
            &mut self,
            request: impl tonic::IntoRequest<super::PrefixValueRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::PrefixValueResponse>>,
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
                "/penumbra.core.app.v1alpha1.QueryService/PrefixValue",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.app.v1alpha1.QueryService",
                        "PrefixValue",
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
        /// Gets the app parameters.
        async fn app_parameters(
            &self,
            request: tonic::Request<super::AppParametersRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AppParametersResponse>,
            tonic::Status,
        >;
        /// General-purpose key-value state query API, that can be used to query
        /// arbitrary keys in the JMT storage.
        async fn key_value(
            &self,
            request: tonic::Request<super::KeyValueRequest>,
        ) -> std::result::Result<
            tonic::Response<super::KeyValueResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the PrefixValue method.
        type PrefixValueStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::PrefixValueResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// General-purpose prefixed key-value state query API, that can be used to query
        /// arbitrary prefixes in the JMT storage.
        /// Returns a stream of `PrefixValueResponse`s.
        async fn prefix_value(
            &self,
            request: tonic::Request<super::PrefixValueRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::PrefixValueStream>,
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
                "/penumbra.core.app.v1alpha1.QueryService/KeyValue" => {
                    #[allow(non_camel_case_types)]
                    struct KeyValueSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::key_value(&inner, request).await
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
                        let method = KeyValueSvc(inner);
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
                "/penumbra.core.app.v1alpha1.QueryService/PrefixValue" => {
                    #[allow(non_camel_case_types)]
                    struct PrefixValueSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<super::PrefixValueRequest>
                    for PrefixValueSvc<T> {
                        type Response = super::PrefixValueResponse;
                        type ResponseStream = T::PrefixValueStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PrefixValueRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::prefix_value(&inner, request).await
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
                        let method = PrefixValueSvc(inner);
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
        const NAME: &'static str = "penumbra.core.app.v1alpha1.QueryService";
    }
}
