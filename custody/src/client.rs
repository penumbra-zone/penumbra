use anyhow::Result;
use penumbra_proto::custody::v1alpha1::custody_protocol_client::CustodyProtocolClient;
use penumbra_transaction::AuthorizationData;
use tonic::async_trait;

use crate::AuthorizeRequest;

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
/// This trait is a wrapper around the proto-generated [`CustodyProtocolClient`] that serves two goals:
///
/// 1. It works on domain types rather than proto-generated types, avoiding conversions;
/// 2. It's easier to write as a trait bound than the `CustodyProtocolClient`,
///   which requires complex bounds on its inner type to enforce that it is a
///   tower `Service`
#[async_trait(?Send)]
pub trait CustodyClient: Sized {
    /// Requests authorization of the transaction with the given description.
    async fn authorize(&mut self, request: AuthorizeRequest) -> Result<AuthorizationData>;
}

// We need to tell `async_trait` not to add a `Send` bound to the boxed
// futures it generates, because the underlying `CustodyProtocolClient` isn't `Sync`,
// but its `authorize` method takes `&mut self`. This would normally cause a huge
// amount of problems, because non-`Send` futures don't compose well, but as long
// as we're calling the method within an async block on a local mutable variable,
// it should be fine.
#[async_trait(?Send)]
impl<T> CustodyClient for CustodyProtocolClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: tonic::codegen::Body + Send + 'static,
    T::Error: Into<tonic::codegen::StdError>,
    <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    async fn authorize(&mut self, request: AuthorizeRequest) -> Result<AuthorizationData> {
        let rsp: AuthorizationData = self
            .authorize(tonic::Request::new(request.into()))
            .await?
            .into_inner()
            .try_into()?;
        Ok(rsp)
    }
}
