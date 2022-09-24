/// Generated client implementations.
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the ibc/channel Msg service.
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
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
    impl<T> MsgClient<T>
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
        ) -> MsgClient<InterceptedService<T, F>>
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
            MsgClient::new(InterceptedService::new(inner, interceptor))
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
        /// ChannelOpenInit defines a rpc handler method for MsgChannelOpenInit.
        pub async fn channel_open_init(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInitResponse,
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
                "/ibc.core.channel.v1.Msg/ChannelOpenInit",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ChannelOpenTry defines a rpc handler method for MsgChannelOpenTry.
        pub async fn channel_open_try(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTryResponse,
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
                "/ibc.core.channel.v1.Msg/ChannelOpenTry",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ChannelOpenAck defines a rpc handler method for MsgChannelOpenAck.
        pub async fn channel_open_ack(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAckResponse,
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
                "/ibc.core.channel.v1.Msg/ChannelOpenAck",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ChannelOpenConfirm defines a rpc handler method for MsgChannelOpenConfirm.
        pub async fn channel_open_confirm(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirmResponse,
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
                "/ibc.core.channel.v1.Msg/ChannelOpenConfirm",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ChannelCloseInit defines a rpc handler method for MsgChannelCloseInit.
        pub async fn channel_close_init(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInitResponse,
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
                "/ibc.core.channel.v1.Msg/ChannelCloseInit",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ChannelCloseConfirm defines a rpc handler method for
        /// MsgChannelCloseConfirm.
        pub async fn channel_close_confirm(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirmResponse,
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
                "/ibc.core.channel.v1.Msg/ChannelCloseConfirm",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// RecvPacket defines a rpc handler method for MsgRecvPacket.
        pub async fn recv_packet(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgRecvPacket,
            >,
        ) -> Result<
            tonic::Response<::ibc_proto::ibc::core::channel::v1::MsgRecvPacketResponse>,
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
                "/ibc.core.channel.v1.Msg/RecvPacket",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Timeout defines a rpc handler method for MsgTimeout.
        pub async fn timeout(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgTimeout,
            >,
        ) -> Result<
            tonic::Response<::ibc_proto::ibc::core::channel::v1::MsgTimeoutResponse>,
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
                "/ibc.core.channel.v1.Msg/Timeout",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// TimeoutOnClose defines a rpc handler method for MsgTimeoutOnClose.
        pub async fn timeout_on_close(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnCloseResponse,
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
                "/ibc.core.channel.v1.Msg/TimeoutOnClose",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Acknowledgement defines a rpc handler method for MsgAcknowledgement.
        pub async fn acknowledgement(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgement,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgementResponse,
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
                "/ibc.core.channel.v1.Msg/Acknowledgement",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod msg_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with MsgServer.
    #[async_trait]
    pub trait Msg: Send + Sync + 'static {
        /// ChannelOpenInit defines a rpc handler method for MsgChannelOpenInit.
        async fn channel_open_init(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInitResponse,
            >,
            tonic::Status,
        >;
        /// ChannelOpenTry defines a rpc handler method for MsgChannelOpenTry.
        async fn channel_open_try(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTryResponse,
            >,
            tonic::Status,
        >;
        /// ChannelOpenAck defines a rpc handler method for MsgChannelOpenAck.
        async fn channel_open_ack(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAckResponse,
            >,
            tonic::Status,
        >;
        /// ChannelOpenConfirm defines a rpc handler method for MsgChannelOpenConfirm.
        async fn channel_open_confirm(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirmResponse,
            >,
            tonic::Status,
        >;
        /// ChannelCloseInit defines a rpc handler method for MsgChannelCloseInit.
        async fn channel_close_init(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInitResponse,
            >,
            tonic::Status,
        >;
        /// ChannelCloseConfirm defines a rpc handler method for
        /// MsgChannelCloseConfirm.
        async fn channel_close_confirm(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirmResponse,
            >,
            tonic::Status,
        >;
        /// RecvPacket defines a rpc handler method for MsgRecvPacket.
        async fn recv_packet(
            &self,
            request: tonic::Request<::ibc_proto::ibc::core::channel::v1::MsgRecvPacket>,
        ) -> Result<
            tonic::Response<::ibc_proto::ibc::core::channel::v1::MsgRecvPacketResponse>,
            tonic::Status,
        >;
        /// Timeout defines a rpc handler method for MsgTimeout.
        async fn timeout(
            &self,
            request: tonic::Request<::ibc_proto::ibc::core::channel::v1::MsgTimeout>,
        ) -> Result<
            tonic::Response<::ibc_proto::ibc::core::channel::v1::MsgTimeoutResponse>,
            tonic::Status,
        >;
        /// TimeoutOnClose defines a rpc handler method for MsgTimeoutOnClose.
        async fn timeout_on_close(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnCloseResponse,
            >,
            tonic::Status,
        >;
        /// Acknowledgement defines a rpc handler method for MsgAcknowledgement.
        async fn acknowledgement(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgement,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgementResponse,
            >,
            tonic::Status,
        >;
    }
    /// Msg defines the ibc/channel Msg service.
    #[derive(Debug)]
    pub struct MsgServer<T: Msg> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msg> MsgServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
    where
        T: Msg,
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
                "/ibc.core.channel.v1.Msg/ChannelOpenInit" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenInitSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit,
                    > for ChannelOpenInitSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInitResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).channel_open_init(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenInitSvc(inner);
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
                "/ibc.core.channel.v1.Msg/ChannelOpenTry" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenTrySvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry,
                    > for ChannelOpenTrySvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenTry,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).channel_open_try(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenTrySvc(inner);
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
                "/ibc.core.channel.v1.Msg/ChannelOpenAck" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenAckSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck,
                    > for ChannelOpenAckSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAckResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).channel_open_ack(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenAckSvc(inner);
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
                "/ibc.core.channel.v1.Msg/ChannelOpenConfirm" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelOpenConfirmSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm,
                    > for ChannelOpenConfirmSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirmResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).channel_open_confirm(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelOpenConfirmSvc(inner);
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
                "/ibc.core.channel.v1.Msg/ChannelCloseInit" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelCloseInitSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit,
                    > for ChannelCloseInitSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInitResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).channel_close_init(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelCloseInitSvc(inner);
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
                "/ibc.core.channel.v1.Msg/ChannelCloseConfirm" => {
                    #[allow(non_camel_case_types)]
                    struct ChannelCloseConfirmSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm,
                    > for ChannelCloseConfirmSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirmResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgChannelCloseConfirm,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).channel_close_confirm(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ChannelCloseConfirmSvc(inner);
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
                "/ibc.core.channel.v1.Msg/RecvPacket" => {
                    #[allow(non_camel_case_types)]
                    struct RecvPacketSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgRecvPacket,
                    > for RecvPacketSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgRecvPacketResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgRecvPacket,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).recv_packet(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RecvPacketSvc(inner);
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
                "/ibc.core.channel.v1.Msg/Timeout" => {
                    #[allow(non_camel_case_types)]
                    struct TimeoutSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgTimeout,
                    > for TimeoutSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgTimeoutResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgTimeout,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).timeout(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TimeoutSvc(inner);
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
                "/ibc.core.channel.v1.Msg/TimeoutOnClose" => {
                    #[allow(non_camel_case_types)]
                    struct TimeoutOnCloseSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose,
                    > for TimeoutOnCloseSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnCloseResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).timeout_on_close(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TimeoutOnCloseSvc(inner);
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
                "/ibc.core.channel.v1.Msg/Acknowledgement" => {
                    #[allow(non_camel_case_types)]
                    struct AcknowledgementSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgement,
                    > for AcknowledgementSvc<T> {
                        type Response = ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgementResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::channel::v1::MsgAcknowledgement,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).acknowledgement(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AcknowledgementSvc(inner);
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
    impl<T: Msg> Clone for MsgServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Msg> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
        const NAME: &'static str = "ibc.core.channel.v1.Msg";
    }
}
