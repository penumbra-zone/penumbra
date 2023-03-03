use anyhow::Result;
use futures::FutureExt;
use penumbra_proto::custody::v1alpha1::custody_protocol_service_client::CustodyProtocolServiceClient;
use penumbra_proto::custody::v1alpha1::AuthorizeResponse;
use std::{future::Future, pin::Pin};

use tonic::codegen::Bytes;

use crate::AuthorizeRequest;

/// A well-typed wrapper around the GRPC custody protocol that uses Rust domain types rather than proto types.
///
/// The custody protocol is used by a wallet client to request authorization for
/// a transaction they’ve constructed.
///
/// Modeling transaction authorization as an asynchronous RPC call encourages
/// software to be written in a way that has a compatible data flow with a “soft
/// HSM”, threshold signing, a hardware wallet, etc.
///
/// The custody protocol does not trust the client to authorize spends, so
/// custody requests must contain sufficient information for the custodian to
/// understand the transaction and determine whether or not it should be
/// authorized.
///
/// This trait is a wrapper around the proto-generated [`CustodyProtocolServiceClient`] that serves two goals:
///
/// 1. It works on domain types rather than proto-generated types, avoiding conversions;
/// 2. It's easier to write as a trait bound than the `CustodyProtocolServiceClient`,
///   which requires complex bounds on its inner type to enforce that it is a
///   tower `Service`
pub trait CustodyClient {
    /// Requests authorization of the transaction with the given description.
    fn authorize(
        &mut self,
        request: AuthorizeRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AuthorizeResponse>> + Send + 'static>>;
}

impl<T> CustodyClient for CustodyProtocolServiceClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Clone + 'static,
    T::ResponseBody: tonic::codegen::Body<Data = Bytes> + Send + 'static,
    T::Future: Send + 'static,
    T::Error: Into<tonic::codegen::StdError>,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    fn authorize(
        &mut self,
        request: AuthorizeRequest,
    ) -> Pin<Box<dyn Future<Output = Result<AuthorizeResponse>> + Send + 'static>> {
        let mut self2 = self.clone();
        async move {
            Ok(self2
                .authorize(tonic::Request::new(request.into()))
                .await?
                .into_inner())
        }
        .boxed()
    }
}
