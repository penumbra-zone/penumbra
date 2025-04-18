use anyhow::anyhow;
use async_stream::try_stream;
use clap::Args;
use futures::{stream::BoxStream, StreamExt as _, TryStreamExt as _};
use penumbra_sdk_asset::asset::{Denom, REGISTRY};
use penumbra_sdk_custody::AuthorizeRequest;
use penumbra_sdk_proto::core::component::sct::v1::EpochByHeightRequest;
use penumbra_sdk_view::Planner;
use rand_core::OsRng;

use crate::clients::Clients;

fn stream_epochs(clients: &Clients) -> BoxStream<'static, anyhow::Result<u64>> {
    let mut view = clients.view();
    let mut sct = clients.sct_query_service();
    try_stream! {
        let mut epoch = None;
        let mut status_stream = view.status_stream().await?;
        while let Some(status) = status_stream.try_next().await? {
            let height = status.latest_known_block_height;
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
        }
    }
    .boxed()
}

#[tracing::instrument(skip(clients))]
async fn vote(clients: &Clients, epoch: u64, denom: Denom) -> anyhow::Result<()> {
    let mut view = clients.view();
    let mut custody = clients.custody();

    let gas_prices = view.gas_prices().await?;

    let rewards_addr = view.address_by_index(Default::default()).await?;
    let voting_notes = view.lqt_voting_notes(epoch, None).await?;

    let mut planner = Planner::new(OsRng);

    planner.set_gas_prices(gas_prices);

    // First, tell the planner to make all the necessary votes.
    planner.lqt_vote(u16::try_from(epoch)?, denom, rewards_addr, &voting_notes);
    // We also want to go ahead and do the consolidation thing,
    // to reduce the number of votes we need in the next epoch.
    // To do so, we need to spend all of these notes, and produce one output per
    // delegator token.
    for note in voting_notes {
        planner.spend(note.note, note.position);
    }

    let plan = planner.plan(view.as_mut(), Default::default()).await?;
    let auth_data = custody
        .authorize(AuthorizeRequest {
            plan: plan.clone(),
            pre_authorizations: Default::default(),
        })
        .await?
        .data
        .expect("auth data should be present")
        .try_into()?;
    tracing::info!("submitting vote");
    let tx = view.witness_and_build(plan, auth_data).await?;
    let tx_id = tx.id().to_string();
    tracing::info!(tx_id, "vote cast");

    Ok(())
}

#[derive(Debug, Args)]
pub struct Opt {
    /// The denom that should continuously be voted for.
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
