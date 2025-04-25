use anyhow::anyhow;
use async_stream::try_stream;
use clap::Args;
use futures::{stream::BoxStream, StreamExt as _, TryStreamExt as _};
use penumbra_sdk_asset::asset::{Denom, REGISTRY};
use penumbra_sdk_proto::core::component::sct::v1::EpochByHeightRequest;
use tokio::time::{Duration, Instant};

use crate::{clients::Clients, planner::build_and_submit};

fn stream_epochs(clients: &Clients) -> BoxStream<'static, anyhow::Result<u64>> {
    const POLL_S: u64 = 30;

    let mut view = clients.view();
    let mut sct = clients.sct_query_service();
    try_stream! {
        let mut epoch = None;
        loop {
            let start = Instant::now();
            let status = view.status().await?;
            if status.catching_up {
                continue;
            }
            let height = status.full_sync_height;
            let current_epoch = sct.epoch_by_height(EpochByHeightRequest { height }).await?.into_inner().epoch.map(|x| x.index);
            match (epoch, current_epoch) {
                (None, Some(x)) => {
                    epoch = Some(x);
                },
                (Some(x), Some(y)) if y > x => {
                    yield y;
                    epoch = Some(y);
                }
                (_, _) => {}
            }
            tokio::time::sleep_until(start + Duration::from_secs(POLL_S)).await;
        }
    }
    .boxed()
}

#[tracing::instrument(skip(clients))]
async fn vote(clients: &Clients, epoch: u64, denom: Denom) -> anyhow::Result<()> {
    let mut view = clients.view();
    tracing::info!("submitting vote");
    let tx = build_and_submit(clients, Default::default(), |mut planner| async {
        let rewards_addr = view.address_by_index(Default::default()).await?;
        let voting_notes = view.lqt_voting_notes(epoch, None).await?;

        // First, tell the planner to make all the necessary votes.
        planner.lqt_vote(u16::try_from(epoch)?, denom, rewards_addr, &voting_notes);
        // We also want to go ahead and do the consolidation thing,
        // to reduce the number of votes we need in the next epoch.
        // To do so, we need to spend all of these notes, and produce one output per
        // delegator token.
        for note in voting_notes {
            planner.spend(note.note, note.position);
        }

        Ok(planner)
    })
    .await?;

    tracing::info!(tx_id = format!("{}", tx.id()), "vote cast");

    Ok(())
}

#[derive(Debug, Args)]
pub struct Opt {
    /// The denom that should continuously be voted for.
    ///
    /// Must be specified as a base denom for an IBC transfer asset, e.g. `transfer/channel-1/uusdc`.
    #[clap(long = "for")]
    denom: String,
}

impl Opt {
    pub async fn run(self, clients: &Clients) -> anyhow::Result<()> {
        let vote_meta = REGISTRY
            .parse_denom(&self.denom)
            .ok_or_else(|| anyhow!("failed to parse denom: '{}'", &self.denom))?;
        let vote_denom = vote_meta.base_denom();
        let mut epochs = stream_epochs(clients);
        while let Some(epoch) = epochs.try_next().await? {
            vote(clients, epoch, vote_denom.clone()).await?;
        }
        Ok(())
    }
}
