//! A basic software key management system that stores keys in memory but
//! presents as an asynchronous signer.

use penumbra_sdk_proto::custody::v1::{self as pb};
use tonic::{async_trait, Request, Response, Status};

/// A "null KMS" that has no keys and errors on any requests.
///
/// Useful when operating in "view-only" mode.
#[derive(Debug, Default)]
pub struct NullKms {}

#[async_trait]
impl pb::custody_service_server::CustodyService for NullKms {
    async fn authorize(
        &self,
        _request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<pb::AuthorizeResponse>, Status> {
        Err(tonic::Status::failed_precondition(
            "Got authorization request in view-only mode to null KMS.",
        ))
    }

    async fn authorize_validator_definition(
        &self,
        _request: Request<pb::AuthorizeValidatorDefinitionRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorDefinitionResponse>, Status> {
        Err(tonic::Status::failed_precondition(
            "Got authorization request in view-only mode to null KMS.",
        ))
    }

    async fn authorize_validator_vote(
        &self,
        _request: Request<pb::AuthorizeValidatorVoteRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorVoteResponse>, Status> {
        Err(tonic::Status::failed_precondition(
            "Got authorization request in view-only mode to null KMS.",
        ))
    }

    async fn export_full_viewing_key(
        &self,
        _request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        Err(tonic::Status::failed_precondition(
            "Got authorization request in view-only mode to null KMS.",
        ))
    }

    async fn confirm_address(
        &self,
        _request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        Err(tonic::Status::failed_precondition(
            "Got authorization request in view-only mode to null KMS.",
        ))
    }
}
