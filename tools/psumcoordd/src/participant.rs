use anyhow::{anyhow, Context, Result};
use penumbra_keys::Address;
use penumbra_proto::{
    penumbra::tools::summoning::v1alpha1::{
        self as pb,
        participate_request::Msg as RequestMsg,
        participate_response::{ContributeNow, Msg as ResponseMsg, Position},
    },
    tools::summoning::v1alpha1::{
        participate_request::Contribution, participate_response::Confirm, ParticipateRequest,
        ParticipateResponse,
    },
};
use tokio::sync::mpsc;
use tonic::{Status, Streaming};

pub struct Participant {
    address: Address,
    rx: Streaming<pb::ParticipateRequest>,
    tx: mpsc::Sender<Result<pb::ParticipateResponse, Status>>,
}

impl Participant {
    pub fn new(
        address: Address,
        rx: Streaming<pb::ParticipateRequest>,
    ) -> (
        Self,
        mpsc::Receiver<Result<pb::ParticipateResponse, Status>>,
    ) {
        // Chosen through extensive performance benchmarking 8^)
        let (tx, rx_response) = mpsc::channel(10);
        (Self { address, rx, tx }, rx_response)
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn is_live(&self) -> bool {
        !self.tx.is_closed()
    }

    pub fn try_notify(&self, position: u32, connected_participants: u32) -> Result<()> {
        let response = ParticipateResponse {
            msg: Some(ResponseMsg::Position(Position {
                position,
                connected_participants,
                last_slot_bid: None,
                your_bid: None,
            })),
        };
        self.tx.try_send(Ok(response)).with_context(|| {
            "Failed to send notification message over channel to client connection watcher"
        })
    }

    // TODO: Use the actual types we want to use
    #[tracing::instrument(skip(self, parent))]
    pub async fn contribute(&mut self, parent: pb::CeremonyCrs) -> Result<pb::CeremonyCrs> {
        self.tx
            .send(Ok(ParticipateResponse {
                msg: Some(ResponseMsg::ContributeNow(ContributeNow {
                    parent: Some(parent),
                })),
            }))
            .await?;
        let msg = self.rx.message().await?;
        if let Some(ParticipateRequest {
            msg:
                Some(RequestMsg::Contribution(Contribution {
                    updated: Some(updated),
                    update_proofs: _,
                    parent_hashes: _,
                })),
        }) = msg
        {
            Ok(updated)
        } else {
            Err(anyhow!(
                "Participant sent a different message than a contribution message when asked"
            ))
        }
    }

    pub async fn confirm(&mut self, slot: u64) -> Result<()> {
        let response = ParticipateResponse {
            msg: Some(ResponseMsg::Confirm(Confirm { slot })),
        };
        self.tx.send(Ok(response)).await?;
        Ok(())
    }
}
