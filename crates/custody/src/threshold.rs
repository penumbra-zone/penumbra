use anyhow::{anyhow, Result};
use penumbra_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_proto::custody::v1alpha1::{self as pb};
use penumbra_transaction::AuthorizationData;
use tonic::{async_trait, Request, Response, Status};

use crate::AuthorizeRequest;

use self::config::Config;

mod config;
mod sign;

/// A trait abstracting over the kind of terminal interface we expect.
///
/// This is mainly used to accomodate the kind of interaction we have with the CLI
/// interface, but it can also be plugged in with more general backends.
#[async_trait]
pub trait Terminal {
    /// Have a user confirm that they want to sign this transaction.
    ///
    /// In an actual terminal, this should display the transaction in a human readable
    /// form, and then get feedback from the user.
    async fn confirm_transaction(&self, transaction: &TransactionPlan) -> bool;

    /// Push an explanatory message to the terminal.
    ///
    /// This message has no relation to the actual protocol, it just allows explaining
    /// what subsequent data means, and what the user needs to do.
    ///
    /// Backends can replace this with a no-op.
    async fn explain(&self, msg: &str);

    /// Broadcast a message to other users.
    async fn broadcast(&self, data: &str);

    /// Wait for a response from *some* other user, it doesn't matter which.
    ///
    /// This function should not return None spuriously, when it does,
    /// it should continue to return None until a message is broadcast.
    async fn next_response(&self) -> Option<String>;
}

/// A custody backend using threshold signing.  
///
/// This backend is initialized with a full viewing key, but only a share
/// of the spend key, which is not enough to sign on its own. Instead,
/// other signers with the same type of configuration need to cooperate
/// to help produce a signature.
pub struct Threshold {}

impl Threshold {
    /// Try and create the necessary signatures to authorize the transaction plan.
    async fn authorize(&self, _request: AuthorizeRequest) -> Result<AuthorizationData> {
        todo!()
    }

    /// Return the full viewing key.
    fn export_full_viewing_key(&self) -> FullViewingKey {
        todo!()
    }

    /// Get the address associated with an index.
    ///
    /// This is just to match the API of the custody trait.
    fn confirm_address(&self, _index: AddressIndex) -> Address {
        todo!()
    }
}

#[async_trait]
impl pb::custody_protocol_service_server::CustodyProtocolService for Threshold {
    async fn authorize(
        &self,
        request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<pb::AuthorizeResponse>, Status> {
        let request = request
            .into_inner()
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;
        let data = self.authorize(request).await.map_err(|e| {
            Status::internal(format!("Failed to process authorization request: {e}"))
        })?;
        Ok(Response::new(pb::AuthorizeResponse {
            data: Some(data.into()),
        }))
    }

    async fn export_full_viewing_key(
        &self,
        _request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        let fvk = self.export_full_viewing_key();
        Ok(Response::new(pb::ExportFullViewingKeyResponse {
            full_viewing_key: Some(fvk.into()),
        }))
    }

    async fn confirm_address(
        &self,
        request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        let index = request
            .into_inner()
            .address_index
            .ok_or(anyhow!("ConfirmAddressRequest missing address_index"))
            .and_then(|x| x.try_into())
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;
        let address = self.confirm_address(index);
        Ok(Response::new(pb::ConfirmAddressResponse {
            address: Some(address.into()),
        }))
    }
}
