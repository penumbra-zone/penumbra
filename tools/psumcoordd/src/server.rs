use penumbra_keys::Address;
use penumbra_proto::penumbra::tools::summoning::v1alpha1::{
    self as pb, ceremony_coordinator_service_server as server,
    participate_request::{Identify, Msg},
};
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{async_trait, codegen::futures_core::Stream, Request, Response, Status, Streaming};

use crate::{participant::Participant, storage::Storage};

#[derive(Clone)]
pub struct CoordinatorService {
    storage: Storage,
    participant_tx: mpsc::Sender<Participant>,
}

impl CoordinatorService {
    pub fn new(storage: Storage, participant_tx: mpsc::Sender<Participant>) -> Self {
        Self {
            storage,
            participant_tx
        }
    }
}

#[async_trait]
impl server::CeremonyCoordinatorService for CoordinatorService {
    type ParticipateStream = ReceiverStream<Result<pb::ParticipateResponse, Status>>;

    async fn participate(
        &self,
        request: Request<Streaming<pb::ParticipateRequest>>,
    ) -> Result<Response<Self::ParticipateStream>, Status> {
        let mut streaming = request.into_inner();
        let address = if let Some(pb::ParticipateRequest {
            msg:
                Some(Msg::Identify(Identify {
                    address: Some(address),
                })),
        }) = streaming.message().await?
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
        self.storage.can_contribute(address).await.map_err(|e| {
            Status::permission_denied(format!("nyo contwibution *cries* fow you {:#}", e))
        })?;
        let (participant, response_rx) = Participant::new(address, streaming);
        self.participant_tx
            .send(participant)
            .await
            .map_err(|e| Status::internal(format!("cannot register participant {:#}", e)))?;

        Ok(Response::new(ReceiverStream::new(response_rx)))
    }
}
