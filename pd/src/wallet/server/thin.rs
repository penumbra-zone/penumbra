use crate::components::{app::View, staking::View as _};
use penumbra_proto::{
    self as proto,
    chain::NoteSource,
    crypto::NoteCommitment,
    thin_client::{thin_protocol_server::ThinProtocol, ValidatorStatusRequest},
};

use tonic::Status;
use tracing::instrument;

struct WalletOverlay<T: View>(T);

#[tonic::async_trait]
impl<T> ThinProtocol for WalletOverlay<T>
where
    T: View,
{
    #[instrument(skip(self, _request))]
    async fn transaction_by_note(
        &self,
        _request: tonic::Request<NoteCommitment>,
    ) -> Result<tonic::Response<NoteSource>, Status> {
        unimplemented!()
    }

    #[instrument(skip(self, request))]
    async fn validator_status(
        &self,
        request: tonic::Request<ValidatorStatusRequest>,
    ) -> Result<tonic::Response<proto::stake::ValidatorStatus>, Status> {
        self.0.check_chain_id(&request.get_ref().chain_id).await?;

        todo!()
    }

    #[instrument(skip(self, request))]
    async fn next_validator_rate(
        &self,
        request: tonic::Request<proto::stake::IdentityKey>,
    ) -> Result<tonic::Response<proto::stake::RateData>, Status> {
        let identity_key = request
            .into_inner()
            .try_into()
            .map_err(|_| tonic::Status::invalid_argument("invalid identity key"))?;

        let rates = self
            .0
            .next_validator_rate(&identity_key)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let rate = match rates {
            Some(x) => x,
            None => return Err(tonic::Status::not_found("rate not found")),
        };

        Ok(tonic::Response::new(rate.into()))
    }
}
