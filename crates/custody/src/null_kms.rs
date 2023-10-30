//! A basic software key management system that stores keys in memory but
//! presents as an asynchronous signer.

use penumbra_proto::custody::v1alpha1::{self as pb, AuthorizeResponse};
use tonic::{async_trait, Request, Response, Status};

/// A "null KMS" that has no keys and errors on any requests.
///
/// Useful when operating in "view-only" mode.
#[derive(Debug, Default)]
pub struct NullKms {}

#[async_trait]
impl pb::custody_protocol_service_server::CustodyProtocolService for NullKms {
    async fn authorize(
        &self,
        _request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        Err(tonic::Status::failed_precondition(
            "Got authorization request in view-only mode to null KMS.",
        ))
    }
}
