use crate::App;
use anyhow::{Context, Result};
use penumbra_asset::Value;
use penumbra_keys::{keys::AddressIndex, Address};
use penumbra_proof_setup::all::{
    Phase1CeremonyContribution, Phase1RawCeremonyCRS, Phase2CeremonyContribution,
    Phase2RawCeremonyCRS,
};
use penumbra_proto::{
    penumbra::tools::summoning::v1alpha1::ceremony_coordinator_service_client::CeremonyCoordinatorServiceClient,
    tools::summoning::v1alpha1::{
        participate_request::{Identify, Msg as RequestMsg},
        participate_response::{Confirm, ContributeNow, Msg as ResponseMsg},
        ParticipateRequest, ParticipateResponse,
    },
    view::v1alpha1::GasPricesRequest,
};
use penumbra_transaction::memo::MemoPlaintext;
use penumbra_view::Planner;
use rand_core::OsRng;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

fn max_message_size(phase: u8) -> usize {
    match phase {
        1 => 200 * 1024 * 1024,
        _ => 100 * 1024 * 1024,
    }
}

#[tracing::instrument(skip(app))]
async fn handle_bid(app: &mut App, to: Address, from: AddressIndex, bid: &str) -> Result<()> {
    let gas_prices = app
        .view
        .as_mut()
        .context("view service must be initialized")?
        .gas_prices(GasPricesRequest {})
        .await?
        .into_inner()
        .gas_prices
        .expect("gas prices must be available")
        .try_into()?;

    let value = bid.parse::<Value>()?;

    let memo_plaintext = MemoPlaintext {
        sender: app.fvk.payment_address(from).0,
        text: "E PLURIBUS UNUM".to_owned(),
    };

    let mut planner = Planner::new(OsRng);
    planner.set_gas_prices(gas_prices);
    planner.output(value, to);
    let plan = planner
        .memo(memo_plaintext)?
        .plan(
            app.view
                .as_mut()
                .context("view service must be initialized")?,
            app.fvk.wallet_id(),
            from,
        )
        .await
        .context("can't build send transaction")?;
    app.build_and_submit_transaction(plan).await?;
    Ok(())
}

#[derive(Debug, clap::Subcommand)]
pub enum CeremonyCmd {
    /// Contribute to the ceremony
    Contribute {
        #[clap(long)]
        phase: u8,
        #[clap(long)]
        coordinator_url: Url,
        #[clap(long)]
        coordinator_address: Address,
        #[clap(long)]
        bid: String,
    },
}

impl CeremonyCmd {
    #[tracing::instrument(skip(self, app))]
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            CeremonyCmd::Contribute {
                phase,
                coordinator_url,
                coordinator_address,
                bid,
            } => {
                println!("¸,ø¤º°` initiating summoning participation `°º¤ø,¸");
                println!("submitting bid for contribution slot: {}", bid);
                if *phase != 1 && *phase != 2 {
                    anyhow::bail!("phase must be 1 or 2.");
                }
                let index = AddressIndex {
                    account: 0,
                    randomizer: b"ceremonyaddr".as_slice().try_into().unwrap(),
                };
                handle_bid(app, *coordinator_address, index, bid).await?;
                let address = app.fvk.payment_address(index).0;

                println!("connecting to coordinator...");
                // After we bid, we need to wait a couple of seconds just for the transaction to be
                // picked up by the coordinator. Else, there is a race wherein the coordinator will kick the
                // client out of the queue because it doesn't see the transaction yet.
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;

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
                    CeremonyCoordinatorServiceClient::connect(coordinator_url.to_string())
                        .await?
                        .max_decoding_message_size(max_message_size(*phase))
                        .max_encoding_message_size(max_message_size(*phase));
                println!("connected to coordinator...");

                use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
                let progress_bar = ProgressBar::with_draw_target(1, ProgressDrawTarget::stdout())
                    .with_style(ProgressStyle::default_bar().template(
                        "[{elapsed}] {bar:50.cyan/blue} {pos:>7}/{len:7} {per_sec} ETA: {eta}",
                    ));
                progress_bar.set_position(0);

                let mut response_rx = client
                    .participate(ReceiverStream::new(req_rx))
                    .await?
                    .into_inner();
                let unparsed_parent = loop {
                    match response_rx.message().await? {
                        None => {
                            progress_bar.abandon();
                            anyhow::bail!("Coordinator closed connection")
                        }
                        Some(ParticipateResponse {
                            msg: Some(ResponseMsg::Position(p)),
                        }) => {
                            // This makes the progress bar go to 100% at the moment we're ready.
                            let len = p.connected_participants;
                            let pos = p.connected_participants - p.position;
                            progress_bar.set_length(len as u64);
                            progress_bar.set_position(pos as u64);
                        }
                        Some(ParticipateResponse {
                            msg:
                                Some(ResponseMsg::ContributeNow(ContributeNow {
                                    parent: Some(parent),
                                })),
                        }) => {
                            progress_bar.finish();
                            break parent;
                        }
                        m => {
                            progress_bar.abandon();
                            anyhow::bail!("Received unexpected  message from coordinator: {:?}", m)
                        }
                    }
                };
                println!("preparing contribution...");
                let contribution = if *phase == 1 {
                    let parent = Phase1RawCeremonyCRS::unchecked_from_protobuf(unparsed_parent)?
                        .assume_valid();
                    let contribution = Phase1CeremonyContribution::make(&parent);
                    contribution.try_into()?
                } else {
                    let parent = Phase2RawCeremonyCRS::unchecked_from_protobuf(unparsed_parent)?
                        .assume_valid();
                    let contribution = Phase2CeremonyContribution::make(&parent);
                    contribution.try_into()?
                };
                println!("submitting contribution...");

                req_tx
                    .send(ParticipateRequest {
                        msg: Some(RequestMsg::Contribution(contribution)),
                    })
                    .await?;
                match response_rx.message().await? {
                    None => anyhow::bail!("Coordinator closed connection"),
                    Some(ParticipateResponse {
                        msg: Some(ResponseMsg::Confirm(Confirm { slot })),
                    }) => {
                        println!("contribution confirmed at slot {slot}");
                        println!("thank you for your help summoning penumbra <3");
                    }
                    m => {
                        anyhow::bail!("Received unexpected message from coordinator: {:?}", m)
                    }
                }

                Ok(())
            }
        }
    }
}
