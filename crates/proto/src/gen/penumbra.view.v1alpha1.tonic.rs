// @generated
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod view_protocol_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ViewProtocolServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ViewProtocolServiceClient<tonic::transport::Channel> {
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
    impl<T> ViewProtocolServiceClient<T>
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
        ) -> ViewProtocolServiceClient<InterceptedService<T, F>>
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
            ViewProtocolServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn status(
            &mut self,
            request: impl tonic::IntoRequest<super::StatusRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Status",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn status_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::StatusStreamRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::StatusStreamResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/StatusStream",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn notes(
            &mut self,
            request: impl tonic::IntoRequest<super::NotesRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::NotesResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Notes",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn notes_for_voting(
            &mut self,
            request: impl tonic::IntoRequest<super::NotesForVotingRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::NotesForVotingResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NotesForVoting",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn witness(
            &mut self,
            request: impl tonic::IntoRequest<super::WitnessRequest>,
        ) -> Result<tonic::Response<super::WitnessResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Witness",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn witness_and_build(
            &mut self,
            request: impl tonic::IntoRequest<super::WitnessAndBuildRequest>,
        ) -> Result<tonic::Response<super::WitnessAndBuildResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/WitnessAndBuild",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn assets(
            &mut self,
            request: impl tonic::IntoRequest<super::AssetsRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::AssetsResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Assets",
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
                "/penumbra.view.v1alpha1.ViewProtocolService/ChainParameters",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn fmd_parameters(
            &mut self,
            request: impl tonic::IntoRequest<super::FmdParametersRequest>,
        ) -> Result<tonic::Response<super::FmdParametersResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/FMDParameters",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn address_by_index(
            &mut self,
            request: impl tonic::IntoRequest<super::AddressByIndexRequest>,
        ) -> Result<tonic::Response<super::AddressByIndexResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/AddressByIndex",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn index_by_address(
            &mut self,
            request: impl tonic::IntoRequest<super::IndexByAddressRequest>,
        ) -> Result<tonic::Response<super::IndexByAddressResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/IndexByAddress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn ephemeral_address(
            &mut self,
            request: impl tonic::IntoRequest<super::EphemeralAddressRequest>,
        ) -> Result<tonic::Response<super::EphemeralAddressResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/EphemeralAddress",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn balance_by_address(
            &mut self,
            request: impl tonic::IntoRequest<super::BalanceByAddressRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::BalanceByAddressResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/BalanceByAddress",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn note_by_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::NoteByCommitmentRequest>,
        ) -> Result<tonic::Response<super::NoteByCommitmentResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NoteByCommitment",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn swap_by_commitment(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapByCommitmentRequest>,
        ) -> Result<tonic::Response<super::SwapByCommitmentResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/SwapByCommitment",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn nullifier_status(
            &mut self,
            request: impl tonic::IntoRequest<super::NullifierStatusRequest>,
        ) -> Result<tonic::Response<super::NullifierStatusResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NullifierStatus",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn transaction_info_by_hash(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionInfoByHashRequest>,
        ) -> Result<
            tonic::Response<super::TransactionInfoByHashResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfoByHash",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn transaction_info(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionInfoRequest>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::TransactionInfoResponse>>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfo",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        pub async fn transaction_planner(
            &mut self,
            request: impl tonic::IntoRequest<super::TransactionPlannerRequest>,
        ) -> Result<tonic::Response<super::TransactionPlannerResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionPlanner",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn broadcast_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTransactionRequest>,
        ) -> Result<
            tonic::Response<super::BroadcastTransactionResponse>,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/BroadcastTransaction",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod view_protocol_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ViewProtocolServiceServer.
    #[async_trait]
    pub trait ViewProtocolService: Send + Sync + 'static {
        async fn status(
            &self,
            request: tonic::Request<super::StatusRequest>,
        ) -> Result<tonic::Response<super::StatusResponse>, tonic::Status>;
        /// Server streaming response type for the StatusStream method.
        type StatusStreamStream: futures_core::Stream<
                Item = Result<super::StatusStreamResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn status_stream(
            &self,
            request: tonic::Request<super::StatusStreamRequest>,
        ) -> Result<tonic::Response<Self::StatusStreamStream>, tonic::Status>;
        /// Server streaming response type for the Notes method.
        type NotesStream: futures_core::Stream<
                Item = Result<super::NotesResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn notes(
            &self,
            request: tonic::Request<super::NotesRequest>,
        ) -> Result<tonic::Response<Self::NotesStream>, tonic::Status>;
        /// Server streaming response type for the NotesForVoting method.
        type NotesForVotingStream: futures_core::Stream<
                Item = Result<super::NotesForVotingResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn notes_for_voting(
            &self,
            request: tonic::Request<super::NotesForVotingRequest>,
        ) -> Result<tonic::Response<Self::NotesForVotingStream>, tonic::Status>;
        async fn witness(
            &self,
            request: tonic::Request<super::WitnessRequest>,
        ) -> Result<tonic::Response<super::WitnessResponse>, tonic::Status>;
        async fn witness_and_build(
            &self,
            request: tonic::Request<super::WitnessAndBuildRequest>,
        ) -> Result<tonic::Response<super::WitnessAndBuildResponse>, tonic::Status>;
        /// Server streaming response type for the Assets method.
        type AssetsStream: futures_core::Stream<
                Item = Result<super::AssetsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn assets(
            &self,
            request: tonic::Request<super::AssetsRequest>,
        ) -> Result<tonic::Response<Self::AssetsStream>, tonic::Status>;
        async fn chain_parameters(
            &self,
            request: tonic::Request<super::ChainParametersRequest>,
        ) -> Result<tonic::Response<super::ChainParametersResponse>, tonic::Status>;
        async fn fmd_parameters(
            &self,
            request: tonic::Request<super::FmdParametersRequest>,
        ) -> Result<tonic::Response<super::FmdParametersResponse>, tonic::Status>;
        async fn address_by_index(
            &self,
            request: tonic::Request<super::AddressByIndexRequest>,
        ) -> Result<tonic::Response<super::AddressByIndexResponse>, tonic::Status>;
        async fn index_by_address(
            &self,
            request: tonic::Request<super::IndexByAddressRequest>,
        ) -> Result<tonic::Response<super::IndexByAddressResponse>, tonic::Status>;
        async fn ephemeral_address(
            &self,
            request: tonic::Request<super::EphemeralAddressRequest>,
        ) -> Result<tonic::Response<super::EphemeralAddressResponse>, tonic::Status>;
        /// Server streaming response type for the BalanceByAddress method.
        type BalanceByAddressStream: futures_core::Stream<
                Item = Result<super::BalanceByAddressResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn balance_by_address(
            &self,
            request: tonic::Request<super::BalanceByAddressRequest>,
        ) -> Result<tonic::Response<Self::BalanceByAddressStream>, tonic::Status>;
        async fn note_by_commitment(
            &self,
            request: tonic::Request<super::NoteByCommitmentRequest>,
        ) -> Result<tonic::Response<super::NoteByCommitmentResponse>, tonic::Status>;
        async fn swap_by_commitment(
            &self,
            request: tonic::Request<super::SwapByCommitmentRequest>,
        ) -> Result<tonic::Response<super::SwapByCommitmentResponse>, tonic::Status>;
        async fn nullifier_status(
            &self,
            request: tonic::Request<super::NullifierStatusRequest>,
        ) -> Result<tonic::Response<super::NullifierStatusResponse>, tonic::Status>;
        async fn transaction_info_by_hash(
            &self,
            request: tonic::Request<super::TransactionInfoByHashRequest>,
        ) -> Result<
            tonic::Response<super::TransactionInfoByHashResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the TransactionInfo method.
        type TransactionInfoStream: futures_core::Stream<
                Item = Result<super::TransactionInfoResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn transaction_info(
            &self,
            request: tonic::Request<super::TransactionInfoRequest>,
        ) -> Result<tonic::Response<Self::TransactionInfoStream>, tonic::Status>;
        async fn transaction_planner(
            &self,
            request: tonic::Request<super::TransactionPlannerRequest>,
        ) -> Result<tonic::Response<super::TransactionPlannerResponse>, tonic::Status>;
        async fn broadcast_transaction(
            &self,
            request: tonic::Request<super::BroadcastTransactionRequest>,
        ) -> Result<tonic::Response<super::BroadcastTransactionResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ViewProtocolServiceServer<T: ViewProtocolService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ViewProtocolService> ViewProtocolServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ViewProtocolServiceServer<T>
    where
        T: ViewProtocolService,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Status" => {
                    #[allow(non_camel_case_types)]
                    struct StatusSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::StatusRequest>
                    for StatusSvc<T> {
                        type Response = super::StatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).status(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StatusSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/StatusStream" => {
                    #[allow(non_camel_case_types)]
                    struct StatusStreamSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::StatusStreamRequest>
                    for StatusStreamSvc<T> {
                        type Response = super::StatusStreamResponse;
                        type ResponseStream = T::StatusStreamStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StatusStreamRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).status_stream(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StatusStreamSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Notes" => {
                    #[allow(non_camel_case_types)]
                    struct NotesSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::NotesRequest>
                    for NotesSvc<T> {
                        type Response = super::NotesResponse;
                        type ResponseStream = T::NotesStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NotesRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).notes(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotesSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NotesForVoting" => {
                    #[allow(non_camel_case_types)]
                    struct NotesForVotingSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::NotesForVotingRequest>
                    for NotesForVotingSvc<T> {
                        type Response = super::NotesForVotingResponse;
                        type ResponseStream = T::NotesForVotingStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NotesForVotingRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).notes_for_voting(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NotesForVotingSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Witness" => {
                    #[allow(non_camel_case_types)]
                    struct WitnessSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::WitnessRequest>
                    for WitnessSvc<T> {
                        type Response = super::WitnessResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WitnessRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).witness(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WitnessSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/WitnessAndBuild" => {
                    #[allow(non_camel_case_types)]
                    struct WitnessAndBuildSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::WitnessAndBuildRequest>
                    for WitnessAndBuildSvc<T> {
                        type Response = super::WitnessAndBuildResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WitnessAndBuildRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).witness_and_build(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WitnessAndBuildSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/Assets" => {
                    #[allow(non_camel_case_types)]
                    struct AssetsSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<super::AssetsRequest>
                    for AssetsSvc<T> {
                        type Response = super::AssetsResponse;
                        type ResponseStream = T::AssetsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AssetsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).assets(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AssetsSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/ChainParameters" => {
                    #[allow(non_camel_case_types)]
                    struct ChainParametersSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
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
                "/penumbra.view.v1alpha1.ViewProtocolService/FMDParameters" => {
                    #[allow(non_camel_case_types)]
                    struct FMDParametersSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::FmdParametersRequest>
                    for FMDParametersSvc<T> {
                        type Response = super::FmdParametersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FmdParametersRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).fmd_parameters(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = FMDParametersSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/AddressByIndex" => {
                    #[allow(non_camel_case_types)]
                    struct AddressByIndexSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::AddressByIndexRequest>
                    for AddressByIndexSvc<T> {
                        type Response = super::AddressByIndexResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddressByIndexRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).address_by_index(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddressByIndexSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/IndexByAddress" => {
                    #[allow(non_camel_case_types)]
                    struct IndexByAddressSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::IndexByAddressRequest>
                    for IndexByAddressSvc<T> {
                        type Response = super::IndexByAddressResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::IndexByAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).index_by_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = IndexByAddressSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/EphemeralAddress" => {
                    #[allow(non_camel_case_types)]
                    struct EphemeralAddressSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::EphemeralAddressRequest>
                    for EphemeralAddressSvc<T> {
                        type Response = super::EphemeralAddressResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EphemeralAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).ephemeral_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EphemeralAddressSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/BalanceByAddress" => {
                    #[allow(non_camel_case_types)]
                    struct BalanceByAddressSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<
                        super::BalanceByAddressRequest,
                    > for BalanceByAddressSvc<T> {
                        type Response = super::BalanceByAddressResponse;
                        type ResponseStream = T::BalanceByAddressStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BalanceByAddressRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).balance_by_address(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BalanceByAddressSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NoteByCommitment" => {
                    #[allow(non_camel_case_types)]
                    struct NoteByCommitmentSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::NoteByCommitmentRequest>
                    for NoteByCommitmentSvc<T> {
                        type Response = super::NoteByCommitmentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NoteByCommitmentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).note_by_commitment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NoteByCommitmentSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/SwapByCommitment" => {
                    #[allow(non_camel_case_types)]
                    struct SwapByCommitmentSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::SwapByCommitmentRequest>
                    for SwapByCommitmentSvc<T> {
                        type Response = super::SwapByCommitmentResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SwapByCommitmentRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).swap_by_commitment(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapByCommitmentSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/NullifierStatus" => {
                    #[allow(non_camel_case_types)]
                    struct NullifierStatusSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::NullifierStatusRequest>
                    for NullifierStatusSvc<T> {
                        type Response = super::NullifierStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::NullifierStatusRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).nullifier_status(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = NullifierStatusSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfoByHash" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionInfoByHashSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::TransactionInfoByHashRequest>
                    for TransactionInfoByHashSvc<T> {
                        type Response = super::TransactionInfoByHashResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionInfoByHashRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_info_by_hash(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionInfoByHashSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionInfo" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionInfoSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::ServerStreamingService<
                        super::TransactionInfoRequest,
                    > for TransactionInfoSvc<T> {
                        type Response = super::TransactionInfoResponse;
                        type ResponseStream = T::TransactionInfoStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionInfoRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_info(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionInfoSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/TransactionPlanner" => {
                    #[allow(non_camel_case_types)]
                    struct TransactionPlannerSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::TransactionPlannerRequest>
                    for TransactionPlannerSvc<T> {
                        type Response = super::TransactionPlannerResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TransactionPlannerRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).transaction_planner(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TransactionPlannerSvc(inner);
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
                "/penumbra.view.v1alpha1.ViewProtocolService/BroadcastTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct BroadcastTransactionSvc<T: ViewProtocolService>(pub Arc<T>);
                    impl<
                        T: ViewProtocolService,
                    > tonic::server::UnaryService<super::BroadcastTransactionRequest>
                    for BroadcastTransactionSvc<T> {
                        type Response = super::BroadcastTransactionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BroadcastTransactionRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).broadcast_transaction(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BroadcastTransactionSvc(inner);
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
    impl<T: ViewProtocolService> Clone for ViewProtocolServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ViewProtocolService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ViewProtocolService> tonic::server::NamedService
    for ViewProtocolServiceServer<T> {
        const NAME: &'static str = "penumbra.view.v1alpha1.ViewProtocolService";
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod view_auth_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ViewAuthServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ViewAuthServiceClient<tonic::transport::Channel> {
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
    impl<T> ViewAuthServiceClient<T>
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
        ) -> ViewAuthServiceClient<InterceptedService<T, F>>
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
            ViewAuthServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn view_auth(
            &mut self,
            request: impl tonic::IntoRequest<super::ViewAuthRequest>,
        ) -> Result<tonic::Response<super::ViewAuthResponse>, tonic::Status> {
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
                "/penumbra.view.v1alpha1.ViewAuthService/ViewAuth",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod view_auth_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ViewAuthServiceServer.
    #[async_trait]
    pub trait ViewAuthService: Send + Sync + 'static {
        async fn view_auth(
            &self,
            request: tonic::Request<super::ViewAuthRequest>,
        ) -> Result<tonic::Response<super::ViewAuthResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct ViewAuthServiceServer<T: ViewAuthService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: ViewAuthService> ViewAuthServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ViewAuthServiceServer<T>
    where
        T: ViewAuthService,
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
                "/penumbra.view.v1alpha1.ViewAuthService/ViewAuth" => {
                    #[allow(non_camel_case_types)]
                    struct ViewAuthSvc<T: ViewAuthService>(pub Arc<T>);
                    impl<
                        T: ViewAuthService,
                    > tonic::server::UnaryService<super::ViewAuthRequest>
                    for ViewAuthSvc<T> {
                        type Response = super::ViewAuthResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ViewAuthRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).view_auth(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ViewAuthSvc(inner);
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
    impl<T: ViewAuthService> Clone for ViewAuthServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: ViewAuthService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: ViewAuthService> tonic::server::NamedService for ViewAuthServiceServer<T> {
        const NAME: &'static str = "penumbra.view.v1alpha1.ViewAuthService";
    }
}
