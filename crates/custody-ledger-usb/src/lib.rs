//! Ledger USB custody for Penumbra.
//!
//! This crate implements Penumbra custody support for Ledger hardware wallets via USB.
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_proto::custody::v1::{self as pb, AuthorizeResponse};
use serde::{Deserialize, Serialize};
use tonic::{async_trait, Request, Response, Status};

/// Options needed to create a new config for custodying with a ledger device.
pub struct InitOptions {}

impl InitOptions {
    pub fn new() -> Self {
        Self {}
    }
}

/// Contains configuration for custody with a ledger device.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Config {}

impl Config {
    /// Initialize custody with a device.
    pub async fn initialize(_opts: InitOptions) -> anyhow::Result<Self> {
        todo!()
    }
}

/// Implements the APIs to allow using
pub struct Service {}

impl Service {
    pub fn new(_config: Config) -> Self {
        todo!()
    }

    /// A convenience method for getting the FVK.
    ///
    /// This also exists by virtue of implementing [`pb::custody_service_server::CustodyService`],
    /// but calling that method is less ergonomic, and ultimately defers to this anyways.
    pub async fn impl_export_full_viewing_key(&self) -> anyhow::Result<FullViewingKey> {
        todo!()
    }
}

#[async_trait]
impl pb::custody_service_server::CustodyService for Service {
    async fn authorize(
        &self,
        _request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        todo!()
    }

    async fn authorize_validator_definition(
        &self,
        _request: Request<pb::AuthorizeValidatorDefinitionRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorDefinitionResponse>, Status> {
        todo!()
    }

    async fn authorize_validator_vote(
        &self,
        _request: Request<pb::AuthorizeValidatorVoteRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorVoteResponse>, Status> {
        todo!()
    }

    async fn export_full_viewing_key(
        &self,
        _request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        todo!()
    }

    async fn confirm_address(
        &self,
        _request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        todo!()
    }
}
