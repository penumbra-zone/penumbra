/// An authorization hash for a Penumbra transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EffectHash {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// Global chain configuration data, such as chain ID, epoch duration, etc.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChainParameters {
    /// The identifier of the chain.
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The duration of each epoch, in number of blocks.
    #[prost(uint64, tag = "2")]
    pub epoch_duration: u64,
}
/// The ratio between two numbers, used in governance to describe vote thresholds and quorums.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ratio {
    /// The numerator.
    #[prost(uint64, tag = "1")]
    pub numerator: u64,
    /// The denominator.
    #[prost(uint64, tag = "2")]
    pub denominator: u64,
}
/// Parameters for Fuzzy Message Detection
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FmdParameters {
    #[prost(uint32, tag = "1")]
    pub precision_bits: u32,
    #[prost(uint64, tag = "2")]
    pub as_of_block_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct KnownAssets {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<
        super::super::super::asset::v1alpha1::DenomMetadata,
    >,
}
/// A spicy transaction ID
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NoteSource {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// A NoteSource paired with the height at which the note was spent
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpendInfo {
    #[prost(message, optional, tag = "1")]
    pub note_source: ::core::option::Option<NoteSource>,
    #[prost(uint64, tag = "2")]
    pub spend_height: u64,
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
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    #[prost(message, optional, tag = "1")]
    pub chain_params: ::core::option::Option<ChainParameters>,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<super::super::stake::v1alpha1::Validator>,
    #[prost(message, repeated, tag = "3")]
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
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Epoch {
    #[prost(uint64, tag = "1")]
    pub index: u64,
    #[prost(uint64, tag = "2")]
    pub start_height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochByHeightRequest {
    #[prost(uint64, tag = "1")]
    pub height: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EpochByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub epoch: ::core::option::Option<Epoch>,
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the chain component.
    #[derive(Debug, Clone)]
    pub struct QueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceClient<tonic::transport::Channel> {
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
        /// TODO: move to SCT cf sct/src/component/view.rs:9 "make epoch management the responsibility of this component"
        pub async fn epoch_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::EpochByHeightRequest>,
        ) -> Result<tonic::Response<super::EpochByHeightResponse>, tonic::Status> {
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
                "/penumbra.core.component.chain.v1alpha1.QueryService/EpochByHeight",
            );
            self.inner.unary(request.into_request(), path, codec).await
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
        /// TODO: move to SCT cf sct/src/component/view.rs:9 "make epoch management the responsibility of this component"
        async fn epoch_by_height(
            &self,
            request: tonic::Request<super::EpochByHeightRequest>,
        ) -> Result<tonic::Response<super::EpochByHeightResponse>, tonic::Status>;
    }
    /// Query operations for the chain component.
    #[derive(Debug)]
    pub struct QueryServiceServer<T: QueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
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
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.core.component.chain.v1alpha1.QueryService/EpochByHeight" => {
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).epoch_by_height(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EpochByHeightSvc(inner);
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
    impl<T: QueryService> Clone for QueryServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: QueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: QueryService> tonic::server::NamedService for QueryServiceServer<T> {
        const NAME: &'static str = "penumbra.core.component.chain.v1alpha1.QueryService";
    }
}
