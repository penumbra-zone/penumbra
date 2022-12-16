#[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Message)]
pub struct AuthorizeRequest {
    /// The transaction plan to authorize.
    #[prost(message, optional, tag = "1")]
    pub plan: ::core::option::Option<super::super::core::transaction::v1alpha1::TransactionPlan>,
    /// Identifies the FVK (and hence the spend authorization key) to use for signing.
    #[prost(message, optional, tag = "2")]
    pub account_id: ::core::option::Option<super::super::core::crypto::v1alpha1::AccountId>,
    /// Optionally, pre-authorization data, if required by the custodian.
    ///
    /// Multiple `PreAuthorization` packets can be included in a single request,
    /// to support multi-party pre-authorizations.
    #[prost(message, repeated, tag = "3")]
    pub pre_authorizations: ::prost::alloc::vec::Vec<PreAuthorization>,
}
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Message)]
pub struct AuthorizeResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<super::super::core::transaction::v1alpha1::AuthorizationData>,
}
/// A pre-authorization packet.  This allows a custodian to delegate (partial)
/// signing authority to other authorization mechanisms.  Details of how a
/// custodian manages those keys are out-of-scope for the custody protocol and
/// are custodian-specific.
#[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Message)]
pub struct PreAuthorization {
    #[prost(oneof = "pre_authorization::PreAuthorization", tags = "1")]
    pub pre_authorization: ::core::option::Option<pre_authorization::PreAuthorization>,
}
/// Nested message and enum types in `PreAuthorization`.
pub mod pre_authorization {
    /// An Ed25519-based preauthorization, containing an Ed25519 signature over the
    /// `TransactionPlan`.
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Message)]
    pub struct Ed25519 {
        /// The Ed25519 verification key used to verify the signature.
        #[prost(bytes = "vec", tag = "1")]
        #[serde(with = "crate::serializers::base64str")]
        pub vk: ::prost::alloc::vec::Vec<u8>,
        /// The Ed25519 signature over the `TransactionPlan`.
        #[prost(bytes = "vec", tag = "2")]
        #[serde(with = "crate::serializers::base64str")]
        pub sig: ::prost::alloc::vec::Vec<u8>,
    }
    #[derive(::serde::Deserialize, ::serde::Serialize, Clone, PartialEq, ::prost::Oneof)]
    pub enum PreAuthorization {
        #[prost(message, tag = "1")]
        Ed25519(Ed25519),
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod custody_protocol_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /// The custody protocol is used by a wallet client to request authorization for
    /// a transaction they've constructed.
    ///
    /// Modeling transaction authorization as an asynchronous RPC call encourages
    /// software to be written in a way that has a compatible data flow with a "soft
    /// HSM", threshold signing, a hardware wallet, etc.
    ///
    /// The custody protocol does not trust the client to authorize spends, so
    /// custody requests must contain sufficient information for the custodian to
    /// understand the transaction and determine whether or not it should be
    /// authorized.
    #[derive(Debug, Clone)]
    pub struct CustodyProtocolServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CustodyProtocolServiceClient<tonic::transport::Channel> {
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
    impl<T> CustodyProtocolServiceClient<T>
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
        ) -> CustodyProtocolServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            CustodyProtocolServiceClient::new(InterceptedService::new(inner, interceptor))
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
        /// Requests authorization of the transaction with the given description.
        pub async fn authorize(
            &mut self,
            request: impl tonic::IntoRequest<super::AuthorizeRequest>,
        ) -> Result<tonic::Response<super::AuthorizeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.custody.v1alpha1.CustodyProtocolService/Authorize",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod custody_protocol_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with CustodyProtocolServiceServer.
    #[async_trait]
    pub trait CustodyProtocolService: Send + Sync + 'static {
        /// Requests authorization of the transaction with the given description.
        async fn authorize(
            &self,
            request: tonic::Request<super::AuthorizeRequest>,
        ) -> Result<tonic::Response<super::AuthorizeResponse>, tonic::Status>;
    }
    /// The custody protocol is used by a wallet client to request authorization for
    /// a transaction they've constructed.
    ///
    /// Modeling transaction authorization as an asynchronous RPC call encourages
    /// software to be written in a way that has a compatible data flow with a "soft
    /// HSM", threshold signing, a hardware wallet, etc.
    ///
    /// The custody protocol does not trust the client to authorize spends, so
    /// custody requests must contain sufficient information for the custodian to
    /// understand the transaction and determine whether or not it should be
    /// authorized.
    #[derive(Debug)]
    pub struct CustodyProtocolServiceServer<T: CustodyProtocolService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: CustodyProtocolService> CustodyProtocolServiceServer<T> {
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CustodyProtocolServiceServer<T>
    where
        T: CustodyProtocolService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.custody.v1alpha1.CustodyProtocolService/Authorize" => {
                    #[allow(non_camel_case_types)]
                    struct AuthorizeSvc<T: CustodyProtocolService>(pub Arc<T>);
                    impl<T: CustodyProtocolService>
                        tonic::server::UnaryService<super::AuthorizeRequest> for AuthorizeSvc<T>
                    {
                        type Response = super::AuthorizeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AuthorizeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).authorize(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AuthorizeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: CustodyProtocolService> Clone for CustodyProtocolServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: CustodyProtocolService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: CustodyProtocolService> tonic::server::NamedService for CustodyProtocolServiceServer<T> {
        const NAME: &'static str = "penumbra.custody.v1alpha1.CustodyProtocolService";
    }
}
