#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParticipateRequest {
    #[prost(oneof = "participate_request::Msg", tags = "1, 2")]
    pub msg: ::core::option::Option<participate_request::Msg>,
}
/// Nested message and enum types in `ParticipateRequest`.
pub mod participate_request {
    /// Sent at the beginning of the connection to identify the participant.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Identify {
        #[prost(message, optional, tag = "1")]
        pub address: ::core::option::Option<
            super::super::super::super::core::keys::v1alpha1::Address,
        >,
    }
    impl ::prost::Name for Identify {
        const NAME: &'static str = "Identify";
        const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.tools.summoning.v1alpha1.ParticipateRequest.{}", Self::NAME
            )
        }
    }
    /// Sent by the participant after getting a `ContributeNow` message.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Contribution {
        #[prost(message, optional, tag = "1")]
        pub updated: ::core::option::Option<super::CeremonyCrs>,
        #[prost(message, optional, tag = "2")]
        pub update_proofs: ::core::option::Option<super::CeremonyLinkingProof>,
        #[prost(message, optional, tag = "3")]
        pub parent_hashes: ::core::option::Option<super::CeremonyParentHashes>,
    }
    impl ::prost::Name for Contribution {
        const NAME: &'static str = "Contribution";
        const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.tools.summoning.v1alpha1.ParticipateRequest.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag = "1")]
        Identify(Identify),
        #[prost(message, tag = "2")]
        Contribution(Contribution),
    }
}
impl ::prost::Name for ParticipateRequest {
    const NAME: &'static str = "ParticipateRequest";
    const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.tools.summoning.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyCrs {
    #[prost(bytes = "vec", tag = "100")]
    pub spend: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "101")]
    pub output: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "102")]
    pub delegator_vote: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "103")]
    pub undelegate_claim: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "104")]
    pub swap: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "105")]
    pub swap_claim: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "106")]
    pub nullifer_derivation_crs: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CeremonyCrs {
    const NAME: &'static str = "CeremonyCrs";
    const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.tools.summoning.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyLinkingProof {
    #[prost(bytes = "vec", tag = "100")]
    pub spend: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "101")]
    pub output: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "102")]
    pub delegator_vote: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "103")]
    pub undelegate_claim: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "104")]
    pub swap: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "105")]
    pub swap_claim: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "106")]
    pub nullifer_derivation_crs: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CeremonyLinkingProof {
    const NAME: &'static str = "CeremonyLinkingProof";
    const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.tools.summoning.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CeremonyParentHashes {
    #[prost(bytes = "vec", tag = "100")]
    pub spend: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "101")]
    pub output: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "102")]
    pub delegator_vote: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "103")]
    pub undelegate_claim: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "104")]
    pub swap: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "105")]
    pub swap_claim: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "106")]
    pub nullifer_derivation_crs: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CeremonyParentHashes {
    const NAME: &'static str = "CeremonyParentHashes";
    const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.tools.summoning.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ParticipateResponse {
    #[prost(oneof = "participate_response::Msg", tags = "1, 2, 3")]
    pub msg: ::core::option::Option<participate_response::Msg>,
}
/// Nested message and enum types in `ParticipateResponse`.
pub mod participate_response {
    /// Streamed to the participant to inform them of their position in the queue.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Position {
        /// The position of the participant in the queue.
        #[prost(uint32, tag = "1")]
        pub position: u32,
        /// The total number of participants in the queue.
        #[prost(uint32, tag = "2")]
        pub connected_participants: u32,
        /// The bid for the most recently executed contribution slot.
        #[prost(message, optional, tag = "3")]
        pub last_slot_bid: ::core::option::Option<
            super::super::super::super::core::num::v1alpha1::Amount,
        >,
        /// The participant's current bid.
        #[prost(message, optional, tag = "4")]
        pub your_bid: ::core::option::Option<
            super::super::super::super::core::num::v1alpha1::Amount,
        >,
    }
    impl ::prost::Name for Position {
        const NAME: &'static str = "Position";
        const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.tools.summoning.v1alpha1.ParticipateResponse.{}", Self::NAME
            )
        }
    }
    /// Sent to the participant to inform them that they should contribute now.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ContributeNow {
        /// The most recent CRS, which the participant should update.
        #[prost(message, optional, tag = "1")]
        pub parent: ::core::option::Option<super::CeremonyCrs>,
    }
    impl ::prost::Name for ContributeNow {
        const NAME: &'static str = "ContributeNow";
        const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.tools.summoning.v1alpha1.ParticipateResponse.{}", Self::NAME
            )
        }
    }
    /// Sent to the participant to confim their contribution was accepted.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Confirm {
        #[prost(uint64, tag = "1")]
        pub slot: u64,
    }
    impl ::prost::Name for Confirm {
        const NAME: &'static str = "Confirm";
        const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.tools.summoning.v1alpha1.ParticipateResponse.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag = "1")]
        Position(Position),
        #[prost(message, tag = "2")]
        ContributeNow(ContributeNow),
        #[prost(message, tag = "3")]
        Confirm(Confirm),
    }
}
impl ::prost::Name for ParticipateResponse {
    const NAME: &'static str = "ParticipateResponse";
    const PACKAGE: &'static str = "penumbra.tools.summoning.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.tools.summoning.v1alpha1.{}", Self::NAME)
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod ceremony_coordinator_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Runs a Phase 2 MPC ceremony with dynamic slot allocation.
    #[derive(Debug, Clone)]
    pub struct CeremonyCoordinatorServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CeremonyCoordinatorServiceClient<tonic::transport::Channel> {
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
    impl<T> CeremonyCoordinatorServiceClient<T>
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
        ) -> CeremonyCoordinatorServiceClient<InterceptedService<T, F>>
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
            CeremonyCoordinatorServiceClient::new(
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
        /// The protocol used to participate in the ceremony.
        ///
        /// The message flow is
        ///
        /// ```text,
        /// Client                     Server
        /// Identify     ===========>
        ///              <=========== Position (repeated)
        ///              <=========== ContributeNow
        /// Contribution ===========>
        ///              <=========== Confirm
        /// ```
        pub async fn participate(
            &mut self,
            request: impl tonic::IntoStreamingRequest<
                Message = super::ParticipateRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ParticipateResponse>>,
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
                "/penumbra.tools.summoning.v1alpha1.CeremonyCoordinatorService/Participate",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.tools.summoning.v1alpha1.CeremonyCoordinatorService",
                        "Participate",
                    ),
                );
            self.inner.streaming(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod ceremony_coordinator_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with CeremonyCoordinatorServiceServer.
    #[async_trait]
    pub trait CeremonyCoordinatorService: Send + Sync + 'static {
        /// Server streaming response type for the Participate method.
        type ParticipateStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::ParticipateResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// The protocol used to participate in the ceremony.
        ///
        /// The message flow is
        ///
        /// ```text,
        /// Client                     Server
        /// Identify     ===========>
        ///              <=========== Position (repeated)
        ///              <=========== ContributeNow
        /// Contribution ===========>
        ///              <=========== Confirm
        /// ```
        async fn participate(
            &self,
            request: tonic::Request<tonic::Streaming<super::ParticipateRequest>>,
        ) -> std::result::Result<
            tonic::Response<Self::ParticipateStream>,
            tonic::Status,
        >;
    }
    /// Runs a Phase 2 MPC ceremony with dynamic slot allocation.
    #[derive(Debug)]
    pub struct CeremonyCoordinatorServiceServer<T: CeremonyCoordinatorService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: CeremonyCoordinatorService> CeremonyCoordinatorServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for CeremonyCoordinatorServiceServer<T>
    where
        T: CeremonyCoordinatorService,
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
                "/penumbra.tools.summoning.v1alpha1.CeremonyCoordinatorService/Participate" => {
                    #[allow(non_camel_case_types)]
                    struct ParticipateSvc<T: CeremonyCoordinatorService>(pub Arc<T>);
                    impl<
                        T: CeremonyCoordinatorService,
                    > tonic::server::StreamingService<super::ParticipateRequest>
                    for ParticipateSvc<T> {
                        type Response = super::ParticipateResponse;
                        type ResponseStream = T::ParticipateStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                tonic::Streaming<super::ParticipateRequest>,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as CeremonyCoordinatorService>::participate(
                                        &inner,
                                        request,
                                    )
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
                        let method = ParticipateSvc(inner);
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
                        let res = grpc.streaming(method, req).await;
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
    impl<T: CeremonyCoordinatorService> Clone for CeremonyCoordinatorServiceServer<T> {
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
    impl<T: CeremonyCoordinatorService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: CeremonyCoordinatorService> tonic::server::NamedService
    for CeremonyCoordinatorServiceServer<T> {
        const NAME: &'static str = "penumbra.tools.summoning.v1alpha1.CeremonyCoordinatorService";
    }
}
