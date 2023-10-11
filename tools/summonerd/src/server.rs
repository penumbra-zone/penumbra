use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proto::penumbra::tools::summoning::v1alpha1::{
    self as pb, ceremony_coordinator_service_server as server,
    participate_request::{Identify, Msg},
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, Request, Response, Status, Streaming};

use crate::{
    participant::Participant,
    penumbra_knower::PenumbraKnower,
    storage::{ContributionAllowed, Storage},
};

#[derive(Clone)]
pub struct CoordinatorService {
    knower: PenumbraKnower,
    storage: Storage,
    participant_tx: mpsc::Sender<(Participant, Amount)>,
}

impl CoordinatorService {
    pub fn new(
        knower: PenumbraKnower,
        storage: Storage,
        participant_tx: mpsc::Sender<(Participant, Amount)>,
    ) -> Self {
        Self {
            knower,
            storage,
            participant_tx,
        }
    }
}

#[async_trait]
impl server::CeremonyCoordinatorService for CoordinatorService {
    type ParticipateStream = ReceiverStream<Result<pb::ParticipateResponse, Status>>;

    #[tracing::instrument(skip(self, request))]
    async fn participate(
        &self,
        request: Request<Streaming<pb::ParticipateRequest>>,
    ) -> Result<Response<Self::ParticipateStream>, Status> {
        let mut streaming = request.into_inner();
        let msg = streaming.message().await?;
        let address = if let Some(pb::ParticipateRequest {
            msg:
                Some(Msg::Identify(Identify {
                    address: Some(address),
                })),
        }) = msg
        {
            tracing::info!(?address, "server connection");
            address
        } else {
            return Err(Status::invalid_argument(
                "Expected first message to be identification with an address",
            ));
        };
        let address = Address::try_from(address)
            .map_err(|e| Status::invalid_argument(format!("Bad address format: {:#}", e)))?;
        // Errors are on our end, None is on their end
        let amount = match self
            .storage
            .can_contribute(&self.knower, &address)
            .await
            .map_err(|e| {
                Status::internal(format!("failed to look up contributor metadata {:#}", e))
            })? {
            ContributionAllowed::Yes(amount) => amount,
            ContributionAllowed::DidntBidEnough(amount) => {
                return Err(Status::permission_denied(format!(
                    "Bid amount {} is not large enough",
                    amount
                )))
            }
            ContributionAllowed::Banned => {
                return Err(Status::permission_denied(format!(
                    "nyo contwibution *cries* fow you"
                )))
            }
            ContributionAllowed::AlreadyContributed => {
                return Err(Status::permission_denied(format!(
                    "Thanks again for your contribution! Participating once is enough to guarantee security, and we'd like to allow other people to participate as well."
                )))
            }
        };
        tracing::info!(?amount, "bid");
        let (participant, response_rx) = Participant::new(address, streaming);
        // TODO: Check if this is what we want to do
        self.participant_tx
            .send((participant, amount))
            .await
            .map_err(|e| Status::internal(format!("cannot register participant {:#}", e)))?;

        Ok(Response::new(ReceiverStream::new(response_rx)))
    }
}
