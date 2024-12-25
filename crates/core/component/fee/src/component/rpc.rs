use async_trait::async_trait;
use cnidarium::Storage;
use penumbra_sdk_proto::core::component::fee::v1::{
    self as pb, query_service_server::QueryService,
};

use super::StateReadExt;

// TODO: Hide this and only expose a Router?
pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl QueryService for Server {
    async fn current_gas_prices(
        &self,
        _request: tonic::Request<pb::CurrentGasPricesRequest>,
    ) -> Result<tonic::Response<pb::CurrentGasPricesResponse>, tonic::Status> {
        let state = self.storage.latest_snapshot();

        let gas_prices = state
            .get_gas_prices()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(pb::CurrentGasPricesResponse {
            gas_prices: Some(gas_prices.into()),
            alt_gas_prices: Vec::new(),
        }))
    }
}
