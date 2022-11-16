use penumbra_chain::StateReadExt as _;
use penumbra_component::dex::StateReadExt as _;
use penumbra_component::shielded_pool::{StateReadExt as _, SupplyRead as _};
use penumbra_component::stake::StateReadExt as _;
use penumbra_crypto::asset::{self, Asset};
use penumbra_proto::{
    self as proto,
    client::v1alpha1::{
        specific_query_service_server::SpecificQueryService, AssetInfoRequest, AssetInfoResponse,
        BatchSwapOutputDataRequest, KeyValueRequest, KeyValueResponse, StubCpmmReservesRequest,
        ValidatorStatusRequest,
    },
    core::{
        chain::v1alpha1::NoteSource,
        crypto::v1alpha1::NoteCommitment,
        dex::v1alpha1::{BatchSwapOutputData, Reserves},
        stake::v1alpha1::ValidatorStatus,
    },
};

use tonic::Status;
use tracing::instrument;

// We need to use the tracing-futures version of Instrument,
// because we want to instrument a Stream, and the Stream trait
// isn't in std, and the tracing::Instrument trait only works with
// (stable) std types.
//use tracing_futures::Instrument;

use super::Info;

#[tonic::async_trait]
impl SpecificQueryService for Info {
    #[instrument(skip(self, request))]
    async fn key_value(
        &self,
        request: tonic::Request<KeyValueRequest>,
    ) -> Result<tonic::Response<KeyValueResponse>, Status> {
        let state = self.storage.latest_state();
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let request = request.into_inner();
        tracing::debug!(?request);

        if request.key.is_empty() {
            return Err(Status::invalid_argument("key is empty"));
        }

        // TODO: how does this align with the ABCI k/v implementation?
        // why do we have two different implementations?
        let (value, proof) = state
            .get_with_proof(request.key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let commitment_proof = ics23::CommitmentProof {
            proof: Some(ics23::commitment_proof::Proof::Exist(proof)),
        };

        Ok(tonic::Response::new(KeyValueResponse {
            value,
            proof: if request.proof {
                Some(commitment_proof)
            } else {
                None
            },
        }))
    }

    #[instrument(skip(self, request))]
    async fn asset_info(
        &self,
        request: tonic::Request<AssetInfoRequest>,
    ) -> Result<tonic::Response<AssetInfoResponse>, Status> {
        let state = self.storage.latest_state();
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let request = request.into_inner();
        let id: asset::Id = request
            .asset_id
            .ok_or_else(|| Status::invalid_argument("missing asset_id"))?
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("could not parse asset_id: {}", e)))?;

        let denom = state
            .denom_by_asset(&id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let rsp = match denom {
            Some(denom) => {
                tracing::debug!(?id, ?denom, "found denom");
                AssetInfoResponse {
                    asset: Some(Asset { id, denom }.into()),
                }
            }
            None => {
                tracing::debug!(?id, "unknown asset id");
                Default::default()
            }
        };

        Ok(tonic::Response::new(rsp))
    }

    #[instrument(skip(self, request))]
    async fn transaction_by_note(
        &self,
        request: tonic::Request<NoteCommitment>,
    ) -> Result<tonic::Response<NoteSource>, Status> {
        let state = self.storage.latest_state();
        let cm = request
            .into_inner()
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid commitment"))?;
        let source = state
            .note_source(cm)
            .await
            .map_err(|e| Status::unavailable(format!("error getting note source: {}", e)))?
            .ok_or_else(|| Status::not_found("note commitment not found"))?;
        tracing::debug!(?cm, ?source);

        Ok(tonic::Response::new(source.into()))
    }

    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<ValidatorStatus>, Status> {
        let state = self.storage.latest_state();
        state.check_chain_id(&request.get_ref().chain_id).await?;

        let id = request
            .into_inner()
            .identity_key
            .ok_or_else(|| Status::invalid_argument("missing identity key"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid identity key"))?;

        let status = state
            .validator_status(&id)
            .await
            .map_err(|e| Status::unavailable(format!("error getting validator status: {}", e)))?
            .ok_or_else(|| Status::not_found("validator not found"))?;

        Ok(tonic::Response::new(status.into()))
    }

    #[instrument(skip(self, request))]
    /// Get the batch swap data associated with a given trading pair and height.
    async fn batch_swap_output_data(
        &self,
        request: tonic::Request<BatchSwapOutputDataRequest>,
    ) -> Result<tonic::Response<BatchSwapOutputData>, Status> {
        let state = self.storage.latest_state();
        let request_inner = request.into_inner();
        let height = request_inner.height;
        let trading_pair = request_inner
            .trading_pair
            .ok_or_else(|| Status::invalid_argument("missing trading_pair"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid trading_pair"))?;

        let output_data = state
            .output_data(height, trading_pair)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match output_data {
            Some(o) => Ok(tonic::Response::new(o.into())),
            None => Err(Status::not_found("batch swap output data not found")),
        }
    }

    #[instrument(skip(self, request))]
    /// Get the batch swap data associated with a given trading pair and height.
    async fn stub_cpmm_reserves(
        &self,
        request: tonic::Request<StubCpmmReservesRequest>,
    ) -> Result<tonic::Response<Reserves>, Status> {
        let state = self.storage.latest_state();
        let request_inner = request.into_inner();
        let trading_pair = request_inner
            .trading_pair
            .ok_or_else(|| Status::invalid_argument("missing trading_pair"))?
            .try_into()
            .map_err(|_| Status::invalid_argument("invalid trading_pair"))?;

        let cpmm_reserves = state
            .stub_cpmm_reserves(&trading_pair)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match cpmm_reserves {
            Some(o) => Ok(tonic::Response::new(o.into())),
            None => Err(Status::not_found("CPMM reserves not found")),
        }
    }

    #[instrument(skip(self, request))]
    async fn next_validator_rate(
        &self,
        request: tonic::Request<proto::core::crypto::v1alpha1::IdentityKey>,
    ) -> Result<tonic::Response<proto::core::stake::v1alpha1::RateData>, Status> {
        let state = self.storage.latest_state();
        let identity_key = request
            .into_inner()
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rate_data = state
            .next_validator_rate(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        match rate_data {
            Some(r) => Ok(tonic::Response::new(r.into())),
            None => Err(Status::not_found("next validator rate not found")),
        }
    }
}
