// @generated
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod oblivious_query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/CompactBlockRange",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn chain_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::ChainParametersRequest>,
        ) -> Result<tonic::Response<super::ChainParametersResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/EpochByHeight",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn validator_info(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorInfoRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::ValidatorInfoResponse>>,
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
        pub async fn info(
            &mut self,
            request: impl tonic::IntoRequest<super::InfoRequest>,
        ) -> Result<tonic::Response<super::InfoResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/Info",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod oblivious_query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ObliviousQueryServiceServer.
    #[async_trait]
    pub trait ObliviousQueryService: Send + Sync + 'static {
        /// Server streaming response type for the CompactBlockRange method.
        type CompactBlockRangeStream: futures_core::Stream<
                Item = Result<super::CompactBlockRangeResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn compact_block_range(
            &self,
            request: tonic::Request<super::CompactBlockRangeRequest>,
        ) -> Result<tonic::Response<Self::CompactBlockRangeStream>, tonic::Status>;
        async fn chain_parameters(
            &self,
            request: tonic::Request<super::ChainParametersRequest>,
        ) -> Result<tonic::Response<super::ChainParametersResponse>, tonic::Status>;
        async fn epoch_by_height(
            &self,
            request: tonic::Request<super::EpochByHeightRequest>,
        ) -> Result<tonic::Response<super::EpochByHeightResponse>, tonic::Status>;
        /// Server streaming response type for the ValidatorInfo method.
        type ValidatorInfoStream: futures_core::Stream<
                Item = Result<super::ValidatorInfoResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn validator_info(
            &self,
            request: tonic::Request<super::ValidatorInfoRequest>,
        ) -> Result<tonic::Response<Self::ValidatorInfoStream>, tonic::Status>;
        async fn info(
            &self,
            request: tonic::Request<super::InfoRequest>,
        ) -> Result<tonic::Response<super::InfoResponse>, tonic::Status>;
    }
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
                    > tonic::server::UnaryService<super::ChainParametersRequest>
                    for ChainParametersSvc<T> {
                        type Response = super::ChainParametersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ChainParametersRequest>,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/EpochByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct EpochByHeightSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/ValidatorInfo" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorInfoSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::ServerStreamingService<super::ValidatorInfoRequest>
                    for ValidatorInfoSvc<T> {
                        type Response = super::ValidatorInfoResponse;
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
                "/penumbra.client.v1alpha1.ObliviousQueryService/Info" => {
                    #[allow(non_camel_case_types)]
                    struct InfoSvc<T: ObliviousQueryService>(pub Arc<T>);
                    impl<
                        T: ObliviousQueryService,
                    > tonic::server::UnaryService<super::InfoRequest> for InfoSvc<T> {
                        type Response = super::InfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).info(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = InfoSvc(inner);
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
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod specific_query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
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
            request: impl tonic::IntoRequest<super::TransactionByNoteRequest>,
        ) -> Result<tonic::Response<super::TransactionByNoteResponse>, tonic::Status> {
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
        ) -> Result<tonic::Response<super::ValidatorStatusResponse>, tonic::Status> {
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
        pub async fn validator_penalty(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidatorPenaltyRequest>,
        ) -> Result<tonic::Response<super::ValidatorPenaltyResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ValidatorPenalty",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn next_validator_rate(
            &mut self,
            request: impl tonic::IntoRequest<super::NextValidatorRateRequest>,
        ) -> Result<tonic::Response<super::NextValidatorRateResponse>, tonic::Status> {
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
        ) -> Result<tonic::Response<super::BatchSwapOutputDataResponse>, tonic::Status> {
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
        pub async fn swap_execution(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapExecutionRequest>,
        ) -> Result<tonic::Response<super::SwapExecutionResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/SwapExecution",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn arb_execution(
            &mut self,
            request: impl tonic::IntoRequest<super::ArbExecutionRequest>,
        ) -> Result<tonic::Response<super::ArbExecutionResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ArbExecution",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn swap_executions(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapExecutionsRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::SwapExecutionsResponse>>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/SwapExecutions",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn arb_executions(
            &mut self,
            request: impl tonic::IntoRequest<super::ArbExecutionsRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::ArbExecutionsResponse>>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ArbExecutions",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn liquidity_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionsRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::LiquidityPositionsResponse>>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositions",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn liquidity_position_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionByIdRequest>,
        ) -> Result<
            tonic::Response<super::LiquidityPositionByIdResponse>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositionById",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn liquidity_positions_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionsByIdRequest>,
        ) -> Result<
            tonic::Response<
                tonic::codec::Streaming<super::LiquidityPositionsByIdResponse>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositionsById",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn liquidity_positions_by_price(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionsByPriceRequest>,
        ) -> Result<
            tonic::Response<
                tonic::codec::Streaming<super::LiquidityPositionsByPriceResponse>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositionsByPrice",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn spread(
            &mut self,
            request: impl tonic::IntoRequest<super::SpreadRequest>,
        ) -> Result<tonic::Response<super::SpreadResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/Spread",
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
        pub async fn proposal_info(
            &mut self,
            request: impl tonic::IntoRequest<super::ProposalInfoRequest>,
        ) -> Result<tonic::Response<super::ProposalInfoResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ProposalInfo",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn proposal_rate_data(
            &mut self,
            request: impl tonic::IntoRequest<super::ProposalRateDataRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::ProposalRateDataResponse>>,
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ProposalRateData",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
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
        pub async fn prefix_value(
            &mut self,
            request: impl tonic::IntoRequest<super::PrefixValueRequest>,
        ) -> Result<
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
                "/penumbra.client.v1alpha1.SpecificQueryService/PrefixValue",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod specific_query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with SpecificQueryServiceServer.
    #[async_trait]
    pub trait SpecificQueryService: Send + Sync + 'static {
        async fn transaction_by_note(
            &self,
            request: tonic::Request<super::TransactionByNoteRequest>,
        ) -> Result<tonic::Response<super::TransactionByNoteResponse>, tonic::Status>;
        async fn validator_status(
            &self,
            request: tonic::Request<super::ValidatorStatusRequest>,
        ) -> Result<tonic::Response<super::ValidatorStatusResponse>, tonic::Status>;
        async fn validator_penalty(
            &self,
            request: tonic::Request<super::ValidatorPenaltyRequest>,
        ) -> Result<tonic::Response<super::ValidatorPenaltyResponse>, tonic::Status>;
        async fn next_validator_rate(
            &self,
            request: tonic::Request<super::NextValidatorRateRequest>,
        ) -> Result<tonic::Response<super::NextValidatorRateResponse>, tonic::Status>;
        async fn batch_swap_output_data(
            &self,
            request: tonic::Request<super::BatchSwapOutputDataRequest>,
        ) -> Result<tonic::Response<super::BatchSwapOutputDataResponse>, tonic::Status>;
        async fn swap_execution(
            &self,
            request: tonic::Request<super::SwapExecutionRequest>,
        ) -> Result<tonic::Response<super::SwapExecutionResponse>, tonic::Status>;
        async fn arb_execution(
            &self,
            request: tonic::Request<super::ArbExecutionRequest>,
        ) -> Result<tonic::Response<super::ArbExecutionResponse>, tonic::Status>;
        /// Server streaming response type for the SwapExecutions method.
        type SwapExecutionsStream: futures_core::Stream<
                Item = Result<super::SwapExecutionsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn swap_executions(
            &self,
            request: tonic::Request<super::SwapExecutionsRequest>,
        ) -> Result<tonic::Response<Self::SwapExecutionsStream>, tonic::Status>;
        /// Server streaming response type for the ArbExecutions method.
        type ArbExecutionsStream: futures_core::Stream<
                Item = Result<super::ArbExecutionsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn arb_executions(
            &self,
            request: tonic::Request<super::ArbExecutionsRequest>,
        ) -> Result<tonic::Response<Self::ArbExecutionsStream>, tonic::Status>;
        /// Server streaming response type for the LiquidityPositions method.
        type LiquidityPositionsStream: futures_core::Stream<
                Item = Result<super::LiquidityPositionsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn liquidity_positions(
            &self,
            request: tonic::Request<super::LiquidityPositionsRequest>,
        ) -> Result<tonic::Response<Self::LiquidityPositionsStream>, tonic::Status>;
        async fn liquidity_position_by_id(
            &self,
            request: tonic::Request<super::LiquidityPositionByIdRequest>,
        ) -> Result<
            tonic::Response<super::LiquidityPositionByIdResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the LiquidityPositionsById method.
        type LiquidityPositionsByIdStream: futures_core::Stream<
                Item = Result<super::LiquidityPositionsByIdResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn liquidity_positions_by_id(
            &self,
            request: tonic::Request<super::LiquidityPositionsByIdRequest>,
        ) -> Result<tonic::Response<Self::LiquidityPositionsByIdStream>, tonic::Status>;
        /// Server streaming response type for the LiquidityPositionsByPrice method.
        type LiquidityPositionsByPriceStream: futures_core::Stream<
                Item = Result<super::LiquidityPositionsByPriceResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn liquidity_positions_by_price(
            &self,
            request: tonic::Request<super::LiquidityPositionsByPriceRequest>,
        ) -> Result<
            tonic::Response<Self::LiquidityPositionsByPriceStream>,
            tonic::Status,
        >;
        async fn spread(
            &self,
            request: tonic::Request<super::SpreadRequest>,
        ) -> Result<tonic::Response<super::SpreadResponse>, tonic::Status>;
        async fn asset_info(
            &self,
            request: tonic::Request<super::AssetInfoRequest>,
        ) -> Result<tonic::Response<super::AssetInfoResponse>, tonic::Status>;
        async fn proposal_info(
            &self,
            request: tonic::Request<super::ProposalInfoRequest>,
        ) -> Result<tonic::Response<super::ProposalInfoResponse>, tonic::Status>;
        /// Server streaming response type for the ProposalRateData method.
        type ProposalRateDataStream: futures_core::Stream<
                Item = Result<super::ProposalRateDataResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn proposal_rate_data(
            &self,
            request: tonic::Request<super::ProposalRateDataRequest>,
        ) -> Result<tonic::Response<Self::ProposalRateDataStream>, tonic::Status>;
        async fn key_value(
            &self,
            request: tonic::Request<super::KeyValueRequest>,
        ) -> Result<tonic::Response<super::KeyValueResponse>, tonic::Status>;
        /// Server streaming response type for the PrefixValue method.
        type PrefixValueStream: futures_core::Stream<
                Item = Result<super::PrefixValueResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn prefix_value(
            &self,
            request: tonic::Request<super::PrefixValueRequest>,
        ) -> Result<tonic::Response<Self::PrefixValueStream>, tonic::Status>;
    }
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
                    > tonic::server::UnaryService<super::TransactionByNoteRequest>
                    for TransactionByNoteSvc<T> {
                        type Response = super::TransactionByNoteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionByNoteRequest>,
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
                        type Response = super::ValidatorStatusResponse;
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ValidatorPenalty" => {
                    #[allow(non_camel_case_types)]
                    struct ValidatorPenaltySvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::ValidatorPenaltyRequest>
                    for ValidatorPenaltySvc<T> {
                        type Response = super::ValidatorPenaltyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidatorPenaltyRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).validator_penalty(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidatorPenaltySvc(inner);
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
                    > tonic::server::UnaryService<super::NextValidatorRateRequest>
                    for NextValidatorRateSvc<T> {
                        type Response = super::NextValidatorRateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NextValidatorRateRequest>,
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
                        type Response = super::BatchSwapOutputDataResponse;
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
                "/penumbra.client.v1alpha1.SpecificQueryService/SwapExecution" => {
                    #[allow(non_camel_case_types)]
                    struct SwapExecutionSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::SwapExecutionRequest>
                    for SwapExecutionSvc<T> {
                        type Response = super::SwapExecutionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SwapExecutionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).swap_execution(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapExecutionSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ArbExecution" => {
                    #[allow(non_camel_case_types)]
                    struct ArbExecutionSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::ArbExecutionRequest>
                    for ArbExecutionSvc<T> {
                        type Response = super::ArbExecutionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ArbExecutionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).arb_execution(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ArbExecutionSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/SwapExecutions" => {
                    #[allow(non_camel_case_types)]
                    struct SwapExecutionsSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::ServerStreamingService<super::SwapExecutionsRequest>
                    for SwapExecutionsSvc<T> {
                        type Response = super::SwapExecutionsResponse;
                        type ResponseStream = T::SwapExecutionsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SwapExecutionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).swap_executions(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapExecutionsSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ArbExecutions" => {
                    #[allow(non_camel_case_types)]
                    struct ArbExecutionsSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::ServerStreamingService<super::ArbExecutionsRequest>
                    for ArbExecutionsSvc<T> {
                        type Response = super::ArbExecutionsResponse;
                        type ResponseStream = T::ArbExecutionsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ArbExecutionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).arb_executions(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ArbExecutionsSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositions" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionsSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::ServerStreamingService<
                        super::LiquidityPositionsRequest,
                    > for LiquidityPositionsSvc<T> {
                        type Response = super::LiquidityPositionsResponse;
                        type ResponseStream = T::LiquidityPositionsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LiquidityPositionsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).liquidity_positions(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionsSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositionById" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionByIdSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::LiquidityPositionByIdRequest>
                    for LiquidityPositionByIdSvc<T> {
                        type Response = super::LiquidityPositionByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LiquidityPositionByIdRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).liquidity_position_by_id(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionByIdSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositionsById" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionsByIdSvc<T: SpecificQueryService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::ServerStreamingService<
                        super::LiquidityPositionsByIdRequest,
                    > for LiquidityPositionsByIdSvc<T> {
                        type Response = super::LiquidityPositionsByIdResponse;
                        type ResponseStream = T::LiquidityPositionsByIdStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LiquidityPositionsByIdRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).liquidity_positions_by_id(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionsByIdSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/LiquidityPositionsByPrice" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionsByPriceSvc<T: SpecificQueryService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::ServerStreamingService<
                        super::LiquidityPositionsByPriceRequest,
                    > for LiquidityPositionsByPriceSvc<T> {
                        type Response = super::LiquidityPositionsByPriceResponse;
                        type ResponseStream = T::LiquidityPositionsByPriceStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::LiquidityPositionsByPriceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).liquidity_positions_by_price(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionsByPriceSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/Spread" => {
                    #[allow(non_camel_case_types)]
                    struct SpreadSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::SpreadRequest>
                    for SpreadSvc<T> {
                        type Response = super::SpreadResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SpreadRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).spread(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SpreadSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ProposalInfo" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalInfoSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::UnaryService<super::ProposalInfoRequest>
                    for ProposalInfoSvc<T> {
                        type Response = super::ProposalInfoResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProposalInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).proposal_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProposalInfoSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/ProposalRateData" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalRateDataSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
                    > tonic::server::ServerStreamingService<
                        super::ProposalRateDataRequest,
                    > for ProposalRateDataSvc<T> {
                        type Response = super::ProposalRateDataResponse;
                        type ResponseStream = T::ProposalRateDataStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ProposalRateDataRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).proposal_rate_data(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProposalRateDataSvc(inner);
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
                "/penumbra.client.v1alpha1.SpecificQueryService/PrefixValue" => {
                    #[allow(non_camel_case_types)]
                    struct PrefixValueSvc<T: SpecificQueryService>(pub Arc<T>);
                    impl<
                        T: SpecificQueryService,
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
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).prefix_value(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PrefixValueSvc(inner);
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
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod tendermint_proxy_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct TendermintProxyServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TendermintProxyServiceClient<tonic::transport::Channel> {
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
    impl<T> TendermintProxyServiceClient<T>
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
        ) -> TendermintProxyServiceClient<InterceptedService<T, F>>
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
            TendermintProxyServiceClient::new(
                InterceptedService::new(inner, interceptor),
            )
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
        pub async fn get_status(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStatusRequest>,
        ) -> Result<tonic::Response<super::GetStatusResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.TendermintProxyService/GetStatus",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn broadcast_tx_async(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTxAsyncRequest>,
        ) -> Result<tonic::Response<super::BroadcastTxAsyncResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.TendermintProxyService/BroadcastTxAsync",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn broadcast_tx_sync(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTxSyncRequest>,
        ) -> Result<tonic::Response<super::BroadcastTxSyncResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.TendermintProxyService/BroadcastTxSync",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxRequest>,
        ) -> Result<tonic::Response<super::GetTxResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.TendermintProxyService/GetTx",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn abci_query(
            &mut self,
            request: impl tonic::IntoRequest<super::AbciQueryRequest>,
        ) -> Result<tonic::Response<super::AbciQueryResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.TendermintProxyService/ABCIQuery",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_block_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockByHeightRequest>,
        ) -> Result<tonic::Response<super::GetBlockByHeightResponse>, tonic::Status> {
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
                "/penumbra.client.v1alpha1.TendermintProxyService/GetBlockByHeight",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod tendermint_proxy_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with TendermintProxyServiceServer.
    #[async_trait]
    pub trait TendermintProxyService: Send + Sync + 'static {
        async fn get_status(
            &self,
            request: tonic::Request<super::GetStatusRequest>,
        ) -> Result<tonic::Response<super::GetStatusResponse>, tonic::Status>;
        async fn broadcast_tx_async(
            &self,
            request: tonic::Request<super::BroadcastTxAsyncRequest>,
        ) -> Result<tonic::Response<super::BroadcastTxAsyncResponse>, tonic::Status>;
        async fn broadcast_tx_sync(
            &self,
            request: tonic::Request<super::BroadcastTxSyncRequest>,
        ) -> Result<tonic::Response<super::BroadcastTxSyncResponse>, tonic::Status>;
        async fn get_tx(
            &self,
            request: tonic::Request<super::GetTxRequest>,
        ) -> Result<tonic::Response<super::GetTxResponse>, tonic::Status>;
        async fn abci_query(
            &self,
            request: tonic::Request<super::AbciQueryRequest>,
        ) -> Result<tonic::Response<super::AbciQueryResponse>, tonic::Status>;
        async fn get_block_by_height(
            &self,
            request: tonic::Request<super::GetBlockByHeightRequest>,
        ) -> Result<tonic::Response<super::GetBlockByHeightResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TendermintProxyServiceServer<T: TendermintProxyService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: TendermintProxyService> TendermintProxyServiceServer<T> {
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
    for TendermintProxyServiceServer<T>
    where
        T: TendermintProxyService,
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
                "/penumbra.client.v1alpha1.TendermintProxyService/GetStatus" => {
                    #[allow(non_camel_case_types)]
                    struct GetStatusSvc<T: TendermintProxyService>(pub Arc<T>);
                    impl<
                        T: TendermintProxyService,
                    > tonic::server::UnaryService<super::GetStatusRequest>
                    for GetStatusSvc<T> {
                        type Response = super::GetStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_status(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetStatusSvc(inner);
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
                "/penumbra.client.v1alpha1.TendermintProxyService/BroadcastTxAsync" => {
                    #[allow(non_camel_case_types)]
                    struct BroadcastTxAsyncSvc<T: TendermintProxyService>(pub Arc<T>);
                    impl<
                        T: TendermintProxyService,
                    > tonic::server::UnaryService<super::BroadcastTxAsyncRequest>
                    for BroadcastTxAsyncSvc<T> {
                        type Response = super::BroadcastTxAsyncResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BroadcastTxAsyncRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).broadcast_tx_async(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTxAsyncSvc(inner);
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
                "/penumbra.client.v1alpha1.TendermintProxyService/BroadcastTxSync" => {
                    #[allow(non_camel_case_types)]
                    struct BroadcastTxSyncSvc<T: TendermintProxyService>(pub Arc<T>);
                    impl<
                        T: TendermintProxyService,
                    > tonic::server::UnaryService<super::BroadcastTxSyncRequest>
                    for BroadcastTxSyncSvc<T> {
                        type Response = super::BroadcastTxSyncResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BroadcastTxSyncRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).broadcast_tx_sync(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTxSyncSvc(inner);
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
                "/penumbra.client.v1alpha1.TendermintProxyService/GetTx" => {
                    #[allow(non_camel_case_types)]
                    struct GetTxSvc<T: TendermintProxyService>(pub Arc<T>);
                    impl<
                        T: TendermintProxyService,
                    > tonic::server::UnaryService<super::GetTxRequest> for GetTxSvc<T> {
                        type Response = super::GetTxResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTxRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_tx(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTxSvc(inner);
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
                "/penumbra.client.v1alpha1.TendermintProxyService/ABCIQuery" => {
                    #[allow(non_camel_case_types)]
                    struct ABCIQuerySvc<T: TendermintProxyService>(pub Arc<T>);
                    impl<
                        T: TendermintProxyService,
                    > tonic::server::UnaryService<super::AbciQueryRequest>
                    for ABCIQuerySvc<T> {
                        type Response = super::AbciQueryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AbciQueryRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).abci_query(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ABCIQuerySvc(inner);
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
                "/penumbra.client.v1alpha1.TendermintProxyService/GetBlockByHeight" => {
                    #[allow(non_camel_case_types)]
                    struct GetBlockByHeightSvc<T: TendermintProxyService>(pub Arc<T>);
                    impl<
                        T: TendermintProxyService,
                    > tonic::server::UnaryService<super::GetBlockByHeightRequest>
                    for GetBlockByHeightSvc<T> {
                        type Response = super::GetBlockByHeightResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetBlockByHeightRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_block_by_height(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetBlockByHeightSvc(inner);
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
    impl<T: TendermintProxyService> Clone for TendermintProxyServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: TendermintProxyService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: TendermintProxyService> tonic::server::NamedService
    for TendermintProxyServiceServer<T> {
        const NAME: &'static str = "penumbra.client.v1alpha1.TendermintProxyService";
    }
}
