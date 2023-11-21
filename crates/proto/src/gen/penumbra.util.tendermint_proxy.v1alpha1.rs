/// GetTxRequest is the request type for the GetTx RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxRequest {
    /// Hash of transaction to retrieve
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    /// Include proofs of the transaction's inclusion in the block
    #[prost(bool, tag = "2")]
    pub prove: bool,
}
impl ::prost::Name for GetTxRequest {
    const NAME: &'static str = "GetTxRequest";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// GetTxResponse is the response type for the GetTx RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTxResponse {
    /// Hash of transaction
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(uint64, tag = "3")]
    pub index: u64,
    #[prost(message, optional, tag = "4")]
    pub tx_result: ::core::option::Option<TxResult>,
    #[prost(bytes = "vec", tag = "5")]
    pub tx: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for GetTxResponse {
    const NAME: &'static str = "GetTxResponse";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxResult {
    #[prost(string, tag = "1")]
    pub log: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub gas_wanted: u64,
    #[prost(uint64, tag = "3")]
    pub gas_used: u64,
    #[prost(message, repeated, tag = "4")]
    pub tags: ::prost::alloc::vec::Vec<Tag>,
}
impl ::prost::Name for TxResult {
    const NAME: &'static str = "TxResult";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tag {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "3")]
    pub index: bool,
}
impl ::prost::Name for Tag {
    const NAME: &'static str = "Tag";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// BroadcastTxAsyncRequest is the request type for the BroadcastTxAsync RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxAsyncRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub params: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub req_id: u64,
}
impl ::prost::Name for BroadcastTxAsyncRequest {
    const NAME: &'static str = "BroadcastTxAsyncRequest";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// BroadcastTxAsyncResponse is the response type for the BroadcastTxAsync RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxAsyncResponse {
    #[prost(uint64, tag = "1")]
    pub code: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BroadcastTxAsyncResponse {
    const NAME: &'static str = "BroadcastTxAsyncResponse";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// BroadcastTxSyncRequest is the request type for the BroadcastTxSync RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxSyncRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub params: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "2")]
    pub req_id: u64,
}
impl ::prost::Name for BroadcastTxSyncRequest {
    const NAME: &'static str = "BroadcastTxSyncRequest";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// BroadcastTxSyncResponse is the response type for the BroadcastTxSync RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BroadcastTxSyncResponse {
    #[prost(uint64, tag = "1")]
    pub code: u64,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for BroadcastTxSyncResponse {
    const NAME: &'static str = "BroadcastTxSyncResponse";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// GetStatusRequest is the request type for the Query/GetStatus RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatusRequest {}
impl ::prost::Name for GetStatusRequest {
    const NAME: &'static str = "GetStatusRequest";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// GetStatusResponse is the response type for the Query/GetStatus RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetStatusResponse {
    #[prost(message, optional, tag = "1")]
    pub node_info: ::core::option::Option<
        super::super::super::super::tendermint::p2p::DefaultNodeInfo,
    >,
    #[prost(message, optional, tag = "2")]
    pub sync_info: ::core::option::Option<SyncInfo>,
    #[prost(message, optional, tag = "3")]
    pub validator_info: ::core::option::Option<
        super::super::super::super::tendermint::types::Validator,
    >,
}
impl ::prost::Name for GetStatusResponse {
    const NAME: &'static str = "GetStatusResponse";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SyncInfo {
    #[prost(bytes = "vec", tag = "1")]
    pub latest_block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub latest_app_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag = "3")]
    pub latest_block_height: u64,
    #[prost(message, optional, tag = "4")]
    pub latest_block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// These are implemented in tendermint, but not
    /// in tendermint-rpc.
    /// bytes earliest_block_hash = 5;
    /// bytes earliest_app_hash = 6;
    /// uint64 earliest_block_height = 7;
    /// google.protobuf.Timestamp earliest_block_time = 8;
    #[prost(bool, tag = "9")]
    pub catching_up: bool,
}
impl ::prost::Name for SyncInfo {
    const NAME: &'static str = "SyncInfo";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// ABCIQueryRequest defines the request structure for the ABCIQuery gRPC query.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciQueryRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "2")]
    pub path: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(bool, tag = "4")]
    pub prove: bool,
}
impl ::prost::Name for AbciQueryRequest {
    const NAME: &'static str = "ABCIQueryRequest";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// ABCIQueryResponse defines the response structure for the ABCIQuery gRPC query.
///
/// Note: This type is a duplicate of the ResponseQuery proto type defined in
/// Tendermint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciQueryResponse {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub index: i64,
    #[prost(bytes = "vec", tag = "6")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "7")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "8")]
    pub proof_ops: ::core::option::Option<
        super::super::super::super::tendermint::crypto::ProofOps,
    >,
    #[prost(int64, tag = "9")]
    pub height: i64,
    #[prost(string, tag = "10")]
    pub codespace: ::prost::alloc::string::String,
}
impl ::prost::Name for AbciQueryResponse {
    const NAME: &'static str = "ABCIQueryResponse";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// GetBlockByHeightRequest is the request type for the Query/GetBlockByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockByHeightRequest {
    #[prost(int64, tag = "1")]
    pub height: i64,
}
impl ::prost::Name for GetBlockByHeightRequest {
    const NAME: &'static str = "GetBlockByHeightRequest";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// GetBlockByHeightResponse is the response type for the Query/GetBlockByHeight RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBlockByHeightResponse {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<
        super::super::super::super::tendermint::types::BlockId,
    >,
    #[prost(message, optional, tag = "2")]
    pub block: ::core::option::Option<
        super::super::super::super::tendermint::types::Block,
    >,
}
impl ::prost::Name for GetBlockByHeightResponse {
    const NAME: &'static str = "GetBlockByHeightResponse";
    const PACKAGE: &'static str = "penumbra.util.tendermint_proxy.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.util.tendermint_proxy.v1alpha1.{}", Self::NAME)
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod tendermint_proxy_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Defines the gRPC query service for proxying requests to an upstream Tendermint RPC.
    #[derive(Debug, Clone)]
    pub struct TendermintProxyServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TendermintProxyServiceClient<tonic::transport::Channel> {
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
        /// Status queries the current status.
        pub async fn get_status(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetStatusResponse>,
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/GetStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService",
                        "GetStatus",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Broadcast a transaction asynchronously.
        pub async fn broadcast_tx_async(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTxAsyncRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTxAsyncResponse>,
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/BroadcastTxAsync",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService",
                        "BroadcastTxAsync",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Broadcast a transaction synchronously.
        pub async fn broadcast_tx_sync(
            &mut self,
            request: impl tonic::IntoRequest<super::BroadcastTxSyncRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTxSyncResponse>,
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/BroadcastTxSync",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService",
                        "BroadcastTxSync",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Fetch a transaction by hash.
        pub async fn get_tx(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTxRequest>,
        ) -> std::result::Result<tonic::Response<super::GetTxResponse>, tonic::Status> {
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/GetTx",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService",
                        "GetTx",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ABCIQuery defines a query handler that supports ABCI queries directly to the
        /// application, bypassing Tendermint completely. The ABCI query must contain
        /// a valid and supported path, including app, custom, p2p, and store.
        pub async fn abci_query(
            &mut self,
            request: impl tonic::IntoRequest<super::AbciQueryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AbciQueryResponse>,
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/ABCIQuery",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService",
                        "ABCIQuery",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// GetBlockByHeight queries block for given height.
        pub async fn get_block_by_height(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBlockByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockByHeightResponse>,
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/GetBlockByHeight",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService",
                        "GetBlockByHeight",
                    ),
                );
            self.inner.unary(req, path, codec).await
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
        /// Status queries the current status.
        async fn get_status(
            &self,
            request: tonic::Request<super::GetStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetStatusResponse>,
            tonic::Status,
        >;
        /// Broadcast a transaction asynchronously.
        async fn broadcast_tx_async(
            &self,
            request: tonic::Request<super::BroadcastTxAsyncRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTxAsyncResponse>,
            tonic::Status,
        >;
        /// Broadcast a transaction synchronously.
        async fn broadcast_tx_sync(
            &self,
            request: tonic::Request<super::BroadcastTxSyncRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BroadcastTxSyncResponse>,
            tonic::Status,
        >;
        /// Fetch a transaction by hash.
        async fn get_tx(
            &self,
            request: tonic::Request<super::GetTxRequest>,
        ) -> std::result::Result<tonic::Response<super::GetTxResponse>, tonic::Status>;
        /// ABCIQuery defines a query handler that supports ABCI queries directly to the
        /// application, bypassing Tendermint completely. The ABCI query must contain
        /// a valid and supported path, including app, custom, p2p, and store.
        async fn abci_query(
            &self,
            request: tonic::Request<super::AbciQueryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::AbciQueryResponse>,
            tonic::Status,
        >;
        /// GetBlockByHeight queries block for given height.
        async fn get_block_by_height(
            &self,
            request: tonic::Request<super::GetBlockByHeightRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetBlockByHeightResponse>,
            tonic::Status,
        >;
    }
    /// Defines the gRPC query service for proxying requests to an upstream Tendermint RPC.
    #[derive(Debug)]
    pub struct TendermintProxyServiceServer<T: TendermintProxyService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
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
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/GetStatus" => {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TendermintProxyService>::get_status(&inner, request)
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
                        let method = GetStatusSvc(inner);
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/BroadcastTxAsync" => {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TendermintProxyService>::broadcast_tx_async(
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
                        let method = BroadcastTxAsyncSvc(inner);
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/BroadcastTxSync" => {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TendermintProxyService>::broadcast_tx_sync(
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
                        let method = BroadcastTxSyncSvc(inner);
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/GetTx" => {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TendermintProxyService>::get_tx(&inner, request).await
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
                        let method = GetTxSvc(inner);
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/ABCIQuery" => {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TendermintProxyService>::abci_query(&inner, request)
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
                        let method = ABCIQuerySvc(inner);
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
                "/penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService/GetBlockByHeight" => {
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
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TendermintProxyService>::get_block_by_height(
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
                        let method = GetBlockByHeightSvc(inner);
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
    impl<T: TendermintProxyService> Clone for TendermintProxyServiceServer<T> {
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
    impl<T: TendermintProxyService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: TendermintProxyService> tonic::server::NamedService
    for TendermintProxyServiceServer<T> {
        const NAME: &'static str = "penumbra.util.tendermint_proxy.v1alpha1.TendermintProxyService";
    }
}
