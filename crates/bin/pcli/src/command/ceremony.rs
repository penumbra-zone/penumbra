use anyhow::{anyhow, Context, Result};
use rand_core::OsRng;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

use penumbra_asset::Value;
use penumbra_keys::{keys::AddressIndex, Address};
use penumbra_num::Amount;
use penumbra_proof_setup::all::{
    Phase1CeremonyContribution, Phase1RawCeremonyCRS, Phase2CeremonyContribution,
    Phase2RawCeremonyCRS,
};
use penumbra_proof_setup::single::log::Hashable;
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

use crate::App;

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

    // If the bid is 0, skip creating a transaction. For instance, this allows reconnecting
    // without paying extra.
    if value.amount == 0u64.into() {
        return Ok(());
    }

    let memo_plaintext = MemoPlaintext::new(
        app.config.full_viewing_key.payment_address(from).0,
        "E PLURIBUS UNUM".into(),
    )?;

    let mut planner = Planner::new(OsRng);
    planner.set_gas_prices(gas_prices);
    planner.output(value, to);
    let plan = planner
        .memo(memo_plaintext)?
        .plan(
            app.view
                .as_mut()
                .context("view service must be initialized")?,
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
        #[clap(long, default_value = "https://summoning.penumbra.zone")]
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

                let index = match *phase {
                    1 => AddressIndex {
                        account: 0,
                        randomizer: b"ceremnyaddr1"
                            .as_slice()
                            .try_into()
                            .expect("12 bytes long"),
                    },
                    2 => AddressIndex {
                        account: 0,
                        randomizer: b"ceremnyaddr2"
                            .as_slice()
                            .try_into()
                            .expect("12 bytes long"),
                    },
                    _ => anyhow::bail!("phase must be 1 or 2."),
                };
                let address = app.config.full_viewing_key.payment_address(index).0;

                println!(
                    "submitting bid {} for contribution slot from address {}",
                    bid, address
                );

                handle_bid(app, *coordinator_address, index, bid).await?;

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
                println!(
                    r#"connected to coordinator!
You may disconnect (CTRL+C) to increase your bid if you don't like your position in the queue.
Otherwise, please keep this window open.
"#
                );
                use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
                let progress_bar = ProgressBar::with_draw_target(1, ProgressDrawTarget::stdout())
                    .with_style(
                        ProgressStyle::default_bar()
                            .template("[{elapsed}] {bar:50.blue/cyan} position {pos} out of {len} connected summoners\t{msg}"),
                    );
                progress_bar.set_position(0);
                progress_bar.enable_steady_tick(1000);

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
                            tracing::debug!(?p);
                            let len = p.connected_participants;
                            // e.g. displaying 1 / 2 instead of 0 / 2
                            let pos = p.position + 1;
                            progress_bar.set_length(len as u64);
                            progress_bar.set_position(pos as u64);
                            progress_bar.set_message(format!(
                                "(your bid: {}, most recent slot bid: {})",
                                Amount::try_from(
                                    p.your_bid.ok_or(anyhow!("expected bid amount"))?
                                )?,
                                Amount::try_from(
                                    p.last_slot_bid.ok_or(anyhow!("expected top bid amount"))?
                                )?
                            ));
                            progress_bar.tick();
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
                println!("preparing contribution... (please keep this window open)");
                let (contribution, hash) = if *phase == 1 {
                    let parent = Phase1RawCeremonyCRS::unchecked_from_protobuf(unparsed_parent)?
                        .assume_valid();
                    let contribution = Phase1CeremonyContribution::make(&parent);
                    let hash = contribution.hash();
                    (contribution.try_into()?, hash)
                } else {
                    let parent = Phase2RawCeremonyCRS::unchecked_from_protobuf(unparsed_parent)?
                        .assume_valid();
                    let contribution = Phase2CeremonyContribution::make(&parent);
                    let hash = contribution.hash();
                    (contribution.try_into()?, hash)
                };
                println!("submitting contribution...");

                req_tx
                    .send(ParticipateRequest {
                        msg: Some(RequestMsg::Contribution(contribution)),
                    })
                    .await?;
                println!("coordinator is validating contribution...");
                match response_rx.message().await? {
                    None => anyhow::bail!("Coordinator closed connection"),
                    Some(ParticipateResponse {
                        msg: Some(ResponseMsg::Confirm(Confirm { slot })),
                    }) => {
                        println!("contribution confirmed at slot {slot}");
                        println!("thank you for your help summoning penumbra <3");
                        println!("here's your contribution receipt (save this to verify inclusion in the final transcript):\n{}", hex::encode_upper(hash.as_ref()));
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
