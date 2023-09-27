use std::time::Duration;

use crate::App;
use anyhow::Result;
use penumbra_proto::{
    penumbra::tools::summoning::v1alpha1::ceremony_coordinator_service_client::CeremonyCoordinatorServiceClient,
    tools::summoning::v1alpha1::{
        participate_request::{Contribution, Identify, Msg as RequestMsg},
        participate_response::{Confirm, ContributeNow, Msg as ResponseMsg},
        ParticipateRequest, ParticipateResponse,
    },
};
use rand_core::OsRng;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

#[derive(Debug, clap::Subcommand)]
pub enum CeremonyCmd {
    /// Contribute to the ceremony
    Contribute {
        #[clap(long)]
        coordinator_url: Url,
        #[clap(long)]
        seconds: u64,
    },
}

impl CeremonyCmd {
    #[tracing::instrument(skip(self, app))]
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            CeremonyCmd::Contribute {
                coordinator_url,
                seconds,
            } => {
                // TODO: Use a fixed address
                let (address, _) = app.fvk.ephemeral_address(
                    &mut OsRng,
                    penumbra_keys::keys::AddressIndex {
                        account: 0,
                        randomizer: b"ceremonyaddr".as_slice().try_into().unwrap(),
                    },
                );
                let (req_tx, req_rx) = mpsc::channel::<ParticipateRequest>(10);
                tracing::debug!(?address, "participate request");
                req_tx
                    .send(ParticipateRequest {
                        msg: Some(RequestMsg::Identify(Identify {
                            address: Some(address.into()),
                        })),
                    })
                    .await?;
                let mut client =
                    CeremonyCoordinatorServiceClient::connect(coordinator_url.to_string()).await?;
                let mut response_rx = client
                    .participate(ReceiverStream::new(req_rx))
                    .await?
                    .into_inner();
                let mut crs = loop {
                    match response_rx.message().await? {
                        None => anyhow::bail!("Coordinator closed connection"),
                        Some(ParticipateResponse {
                            msg: Some(ResponseMsg::Position(p)),
                        }) => {
                            println!("{:?}", p);
                        }
                        Some(ParticipateResponse {
                            msg:
                                Some(ResponseMsg::ContributeNow(ContributeNow {
                                    parent: Some(parent),
                                })),
                        }) => {
                            tracing::debug!("contribute now");
                            break parent;
                        }
                        m => {
                            anyhow::bail!("Received unexpected  message from coordinator: {:?}", m)
                        }
                    }
                };
                // TODO: Make an actual contribution
                crs.spend.push(0xFF);
                tokio::time::sleep(Duration::from_secs(*seconds)).await;
                req_tx
                    .send(ParticipateRequest {
                        msg: Some(RequestMsg::Contribution(Contribution {
                            updated: Some(crs),
                            update_proofs: None,
                            parent_hashes: None,
                        })),
                    })
                    .await?;
                match response_rx.message().await? {
                    None => anyhow::bail!("Coordinator closed connection"),
                    Some(ParticipateResponse {
                        msg: Some(ResponseMsg::Confirm(Confirm { slot })),
                    }) => {
                        println!("Contribution confirmed at slot {slot}");
                    }
                    m => {
                        anyhow::bail!("Received unexpected  message from coordinator: {:?}", m)
                    }
                }

                Ok(())
            }
        }
    }
}
