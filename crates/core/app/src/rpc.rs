use std::pin::Pin;

use futures::{StreamExt, TryStreamExt};
use penumbra_chain::component::{AppHashRead, StateReadExt as _};
use penumbra_dao::StateReadExt as _;
use penumbra_governance::StateReadExt as _;
use penumbra_ibc::StateReadExt as _;
use penumbra_proto::core::app::v1alpha1::{
    key_value_response::Value, query_service_server::QueryService, AppParametersRequest,
    AppParametersResponse, KeyValueRequest, KeyValueResponse, PrefixValueRequest,
    PrefixValueResponse,
};
use penumbra_stake::StateReadExt as _;
use penumbra_storage::{StateRead, Storage};
use tonic::Status;
use tracing::instrument;

use crate::params::AppParameters;

// TODO: Hide this and only expose a Router?
pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    #[instrument(skip(self, request))]
    async fn app_parameters(
        &self,
        request: tonic::Request<AppParametersRequest>,
    ) -> Result<tonic::Response<AppParametersResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| {
                tonic::Status::unknown(format!(
                    "failed to validate chain id during app parameters lookup: {e}"
                ))
            })?;

        let chain_params = state.get_chain_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting chain parameters: {e}"))
        })?;
        let stake_params = state.get_stake_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting stake parameters: {e}"))
        })?;
        let ibc_params = state.get_ibc_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting ibc parameters: {e}"))
        })?;
        let governance_params = state.get_governance_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting governance parameters: {e}"))
        })?;
        let dao_params = state.get_dao_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting dao parameters: {e}"))
        })?;

        Ok(tonic::Response::new(AppParametersResponse {
            app_parameters: Some(
                AppParameters {
                    chain_params,
                    stake_params,
                    ibc_params,
                    governance_params,
                    dao_params,
                }
                .into(),
            ),
        }))
    }

    #[instrument(skip(self, request))]
    async fn key_value(
        &self,
        request: tonic::Request<KeyValueRequest>,
    ) -> Result<tonic::Response<KeyValueResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;

        let request = request.into_inner();
        tracing::debug!(?request);

        if request.key.is_empty() {
            return Err(Status::invalid_argument("key is empty"));
        }

        // TODO(erwan): we are unconditionally generating the proof here; we shouldn't do that if the
        // request doesn't ask for it
        let (some_value, proof) = state
            .get_with_proof_to_apphash(request.key.into_bytes())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(KeyValueResponse {
            value: some_value.map(|value| Value { value }),
            proof: if request.proof {
                Some(ibc_proto::ibc::core::commitment::v1::MerkleProof {
                    proofs: proof
                        .proofs
                        .into_iter()
                        .map(|p| {
                            let mut encoded = Vec::new();
                            prost::Message::encode(&p, &mut encoded).expect("able to encode proof");
                            prost::Message::decode(&*encoded).expect("able to decode proof")
                        })
                        .collect(),
                })
            } else {
                None
            },
        }))
    }

    type PrefixValueStream =
        Pin<Box<dyn futures::Stream<Item = Result<PrefixValueResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn prefix_value(
        &self,
        request: tonic::Request<PrefixValueRequest>,
    ) -> Result<tonic::Response<Self::PrefixValueStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request = request.into_inner();
        tracing::debug!(?request);

        if request.prefix.is_empty() {
            return Err(Status::invalid_argument("prefix is empty"));
        }

        Ok(tonic::Response::new(
            state
                .prefix_raw(&request.prefix)
                .map_ok(|i: (String, Vec<u8>)| {
                    let (key, value) = i;
                    PrefixValueResponse { key, value }
                })
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!(
                        "error getting prefix value from storage: {e}"
                    ))
                })
                // TODO: how do we instrument a Stream
                //.instrument(Span::current())
                .boxed(),
        ))
    }
}
