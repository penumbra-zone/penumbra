/// Generated client implementations.
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the ibc/connection Msg service.
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
        /// ConnectionOpenInit defines a rpc handler method for MsgConnectionOpenInit.
        pub async fn connection_open_init(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInitResponse,
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenInit",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ConnectionOpenTry defines a rpc handler method for MsgConnectionOpenTry.
        pub async fn connection_open_try(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTryResponse,
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenTry",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ConnectionOpenAck defines a rpc handler method for MsgConnectionOpenAck.
        pub async fn connection_open_ack(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAckResponse,
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenAck",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// ConnectionOpenConfirm defines a rpc handler method for
        /// MsgConnectionOpenConfirm.
        pub async fn connection_open_confirm(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirmResponse,
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenConfirm",
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
        /// ConnectionOpenInit defines a rpc handler method for MsgConnectionOpenInit.
        async fn connection_open_init(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInitResponse,
            >,
            tonic::Status,
        >;
        /// ConnectionOpenTry defines a rpc handler method for MsgConnectionOpenTry.
        async fn connection_open_try(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTryResponse,
            >,
            tonic::Status,
        >;
        /// ConnectionOpenAck defines a rpc handler method for MsgConnectionOpenAck.
        async fn connection_open_ack(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAckResponse,
            >,
            tonic::Status,
        >;
        /// ConnectionOpenConfirm defines a rpc handler method for
        /// MsgConnectionOpenConfirm.
        async fn connection_open_confirm(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm,
            >,
        ) -> Result<
            tonic::Response<
                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirmResponse,
            >,
            tonic::Status,
        >;
    }
    /// Msg defines the ibc/connection Msg service.
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenInit" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectionOpenInitSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit,
                    > for ConnectionOpenInitSvc<T> {
                        type Response = ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInitResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).connection_open_init(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectionOpenInitSvc(inner);
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenTry" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectionOpenTrySvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry,
                    > for ConnectionOpenTrySvc<T> {
                        type Response = ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenTry,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).connection_open_try(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectionOpenTrySvc(inner);
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenAck" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectionOpenAckSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck,
                    > for ConnectionOpenAckSvc<T> {
                        type Response = ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAckResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenAck,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).connection_open_ack(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectionOpenAckSvc(inner);
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
                "/ibc.core.connection.v1.Msg/ConnectionOpenConfirm" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectionOpenConfirmSvc<T: Msg>(pub Arc<T>);
                    impl<
                        T: Msg,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm,
                    > for ConnectionOpenConfirmSvc<T> {
                        type Response = ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirmResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).connection_open_confirm(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectionOpenConfirmSvc(inner);
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
        const NAME: &'static str = "ibc.core.connection.v1.Msg";
    }
}
