/// The configuration parameters for the auction component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuctionParameters {}
impl ::prost::Name for AuctionParameters {
    const NAME: &'static str = "AuctionParameters";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
/// Genesis data for the auction component.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisContent {
    /// The configuration parameters for the auction component at genesis.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<AuctionParameters>,
}
impl ::prost::Name for GenesisContent {
    const NAME: &'static str = "GenesisContent";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
/// A unique identifier for an auction, obtained from hashing a domain separator
/// along with the immutable part of an auction description.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuctionId {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for AuctionId {
    const NAME: &'static str = "AuctionId";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
/// A bearer NFT tracking ownership of an auction and its proceeds.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuctionNft {
    #[prost(message, optional, tag = "1")]
    pub id: ::core::option::Option<AuctionId>,
    #[prost(uint64, tag = "2")]
    pub seq: u64,
}
impl ::prost::Name for AuctionNft {
    const NAME: &'static str = "AuctionNft";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
/// Describes a Dutch auction using programmatic liquidity on the DEX.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DutchAuctionDescription {
    /// The value the seller wishes to auction.
    #[prost(message, optional, tag = "1")]
    pub input: ::core::option::Option<super::super::super::asset::v1::Value>,
    /// The asset ID of the target asset the seller wishes to acquire.
    #[prost(message, optional, tag = "2")]
    pub output_id: ::core::option::Option<super::super::super::asset::v1::AssetId>,
    /// The maximum output the seller can receive.
    ///
    /// This implicitly defines the starting price for the auction.
    #[prost(message, optional, tag = "3")]
    pub max_output: ::core::option::Option<super::super::super::num::v1::Amount>,
    /// The minimum output the seller is willing to receive.
    ///
    /// This implicitly defines the ending price for the auction.
    #[prost(message, optional, tag = "4")]
    pub min_output: ::core::option::Option<super::super::super::num::v1::Amount>,
    /// The block height at which the auction begins.
    ///
    /// This allows the seller to schedule an auction at a future time.
    #[prost(uint64, tag = "5")]
    pub start_height: u64,
    /// The block height at which the auction ends.
    ///
    /// Together with `start_height`, `max_output`, and `min_output`,
    /// this implicitly defines the speed of the auction.
    #[prost(uint64, tag = "6")]
    pub end_height: u64,
    /// The number of discrete price steps to use for the auction.
    ///
    /// `end_height - start_height` must be a multiple of `step_count`.
    #[prost(uint64, tag = "7")]
    pub step_count: u64,
    /// A random nonce used to allow identical auctions to have
    /// distinct auction IDs.
    #[prost(bytes = "vec", tag = "8")]
    pub nonce: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for DutchAuctionDescription {
    const NAME: &'static str = "DutchAuctionDescription";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DutchAuctionState {
    /// The sequence number of the auction state.
    ///
    /// Dutch auctions move from:
    /// 0 (opened) => 1 (closed) => n (withdrawn)
    #[prost(uint64, tag = "1")]
    pub seq: u64,
    /// If present, the current position controlled by this auction.
    #[prost(message, optional, tag = "2")]
    pub current_position: ::core::option::Option<super::super::dex::v1::PositionId>,
    /// If present, the next trigger height to step down the price.
    #[prost(uint64, tag = "3")]
    pub next_trigger: u64,
    /// The amount of the input asset directly owned by the auction.
    ///
    /// The auction may also own the input asset indirectly,
    /// via the reserves of `current_position` if it exists.
    #[prost(message, optional, tag = "4")]
    pub input_reserves: ::core::option::Option<super::super::super::num::v1::Amount>,
    /// The amount of the output asset directly owned by the auction.
    ///
    /// The auction may also own the output asset indirectly,
    /// via the reserves of `current_position` if it exists.
    #[prost(message, optional, tag = "5")]
    pub output_reserves: ::core::option::Option<super::super::super::num::v1::Amount>,
}
impl ::prost::Name for DutchAuctionState {
    const NAME: &'static str = "DutchAuctionState";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DutchAuction {
    /// The immutable data describing the auction and its auction ID.
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<DutchAuctionDescription>,
    /// The mutable data describing the auction's execution.
    #[prost(message, optional, tag = "2")]
    pub state: ::core::option::Option<DutchAuctionState>,
}
impl ::prost::Name for DutchAuction {
    const NAME: &'static str = "DutchAuction";
    const PACKAGE: &'static str = "penumbra.core.component.auction.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!(
            "penumbra.core.component.auction.v1alpha1.{}", Self::NAME
        )
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the Auction component.
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
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServiceServer.
    #[async_trait]
    pub trait QueryService: Send + Sync + 'static {}
    /// Query operations for the Auction component.
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
        const NAME: &'static str = "penumbra.core.component.auction.v1alpha1.QueryService";
    }
}
