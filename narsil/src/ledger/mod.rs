//! Logic for enabling `narsild` to interact with chain state.
use penumbra_proto::narsil::v1alpha1::ledger::ledger_service_server::LedgerService;
use tendermint::v0_34::abci;
use tonic::Status;
use tracing::instrument;

use crate::metrics;

pub mod app;
pub mod consensus;
pub mod info;
pub mod mempool;
pub mod snapshot;
pub use info::Info;

/// RAII guard used to increment and decrement an active connection counter.
///
/// This ensures we appropriately decrement the counter when the guard goes out of scope.
struct CompactBlockConnectionCounter {}

impl CompactBlockConnectionCounter {
    pub fn _new() -> Self {
        metrics::increment_gauge!(
            metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS,
            1.0
        );
        CompactBlockConnectionCounter {}
    }
}

impl Drop for CompactBlockConnectionCounter {
    fn drop(&mut self) {
        metrics::decrement_gauge!(
            metrics::CLIENT_OBLIVIOUS_COMPACT_BLOCK_ACTIVE_CONNECTIONS,
            1.0
        );
    }
}

#[tonic::async_trait]
impl LedgerService for Info {
    #[instrument(skip(self, request))]
    async fn info(
        &self,
        request: tonic::Request<penumbra_proto::narsil::v1alpha1::ledger::InfoRequest>,
    ) -> Result<tonic::Response<penumbra_proto::narsil::v1alpha1::ledger::InfoResponse>, Status>
    {
        let info = self
            .info(abci::request::Info {
                version: request.get_ref().version.clone(),
                block_version: request.get_ref().block_version,
                p2p_version: request.get_ref().p2p_version,
                abci_version: request.get_ref().abci_version.clone(),
            })
            .await
            .map_err(|e| tonic::Status::unknown(format!("error getting ABCI info: {e}")))?;

        Ok(tonic::Response::new(
            penumbra_proto::narsil::v1alpha1::ledger::InfoResponse {
                data: info.data.into(),
                version: info.version,
                app_version: info.app_version,
                last_block_height: info.last_block_height.into(),
                last_block_app_hash: info.last_block_app_hash.into(),
            },
        ))
    }
}
