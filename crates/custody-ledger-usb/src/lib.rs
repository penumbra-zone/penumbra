//! Ledger USB custody for Penumbra.
//!
//! This crate implements Penumbra custody support for Ledger hardware wallets via USB.

/// Abstraction layer over the ledger libraries for device interaction.
mod device;

use std::{ops::DerefMut, sync::Arc};

use device::Device;
use penumbra_sdk_custody::AuthorizeRequest;
use penumbra_sdk_keys::{keys::AddressIndex, Address, FullViewingKey};
use penumbra_sdk_proto::custody::v1::{self as pb, AuthorizeResponse};
use penumbra_sdk_transaction::{AuthorizationData, TransactionPlan};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use tonic::{async_trait, Request, Response, Status};

/// Options needed to create a new config for custodying with a ledger device.
#[derive(Default)]
pub struct InitOptions {}

/// Contains configuration for custody with a ledger device.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Config {}

impl Config {
    /// Initialize custody with a device.
    pub async fn initialize(_opts: InitOptions) -> anyhow::Result<Self> {
        // In the future, we might do things like actually create some kind
        // of device checksum or something like that.
        Ok(Self {})
    }
}

/// Implements the APIs to allow using
pub struct Service {
    device: Arc<Mutex<Option<Device>>>,
}

impl Service {
    pub fn new(_config: Config) -> Self {
        Self {
            device: Arc::new(Default::default()),
        }
    }

    /// Acquire a new device, potentially initializing it if necessary.
    ///
    /// The resulting handle can be dereferenced to get a device.
    /// This handle will have exclusive access to the device.
    async fn acquire_device(&self) -> anyhow::Result<impl DerefMut<Target = Device> + '_> {
        let mut guard = self.device.lock().await;
        if guard.is_none() {
            *guard = Some(Device::connect_to_first().await?);
        }
        let out = MutexGuard::map(guard, |x| x.as_mut().expect("device should be initialized"));
        Ok(out)
    }

    /// A convenience method for getting the FVK.
    ///
    /// This also exists by virtue of implementing [`pb::custody_service_server::CustodyService`],
    /// but calling that method is less ergonomic, and ultimately defers to this anyways.
    pub async fn impl_export_full_viewing_key(&self) -> anyhow::Result<FullViewingKey> {
        self.acquire_device().await?.get_fvk().await
    }

    /// A convenience method for confirming an address.
    ///
    /// This will ask the user to confirm the address on their device.
    pub async fn impl_confirm_address(&self, index: AddressIndex) -> anyhow::Result<Address> {
        self.acquire_device().await?.confirm_addr(index).await
    }

    /// A convenience method for authorizing a transaction
    pub async fn impl_authorize(&self, plan: TransactionPlan) -> anyhow::Result<AuthorizationData> {
        self.acquire_device().await?.authorize(plan).await
    }
}

#[async_trait]
impl pb::custody_service_server::CustodyService for Service {
    async fn authorize(
        &self,
        request: Request<pb::AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        let request: AuthorizeRequest = request
            .into_inner()
            .try_into()
            .map_err(|e: anyhow::Error| Status::invalid_argument(e.to_string()))?;

        let authorization_data = self
            .impl_authorize(request.plan)
            .await
            .map_err(|e| Status::unauthenticated(format!("{e:#}")))?;

        let authorization_response = AuthorizeResponse {
            data: Some(authorization_data.into()),
        };

        Ok(Response::new(authorization_response))
    }

    async fn authorize_validator_definition(
        &self,
        _request: Request<pb::AuthorizeValidatorDefinitionRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorDefinitionResponse>, Status> {
        unimplemented!("ledger does not support validator operations")
    }

    async fn authorize_validator_vote(
        &self,
        _request: Request<pb::AuthorizeValidatorVoteRequest>,
    ) -> Result<Response<pb::AuthorizeValidatorVoteResponse>, Status> {
        unimplemented!("ledger does not support validator operations")
    }

    async fn export_full_viewing_key(
        &self,
        _request: Request<pb::ExportFullViewingKeyRequest>,
    ) -> Result<Response<pb::ExportFullViewingKeyResponse>, Status> {
        let fvk = self
            .impl_export_full_viewing_key()
            .await
            .map_err(|e| Status::internal(format!("{}", e)))?;
        Ok(Response::new(pb::ExportFullViewingKeyResponse {
            full_viewing_key: Some(fvk.into()),
        }))
    }

    async fn confirm_address(
        &self,
        request: Request<pb::ConfirmAddressRequest>,
    ) -> Result<Response<pb::ConfirmAddressResponse>, Status> {
        let address_index = request
            .into_inner()
            .address_index
            .ok_or_else(|| {
                Status::invalid_argument("missing address index in confirm address request")
            })?
            .try_into()
            .map_err(|e| {
                Status::invalid_argument(format!(
                    "invalid address index in confirm address request: {e:#}"
                ))
            })?;
        let address = self
            .impl_confirm_address(address_index)
            .await
            .map_err(|e| Status::internal(format!("{}", e)))?;

        Ok(Response::new(pb::ConfirmAddressResponse {
            address: Some(address.into()),
        }))
    }
}
