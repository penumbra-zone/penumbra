use std::collections::BTreeMap;

use anyhow::anyhow;
use penumbra_sdk_asset::{
    asset::{self, REGISTRY},
    Value,
};
use penumbra_sdk_fee::{FeeTier, GasPrices};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::sct::v1::{
    query_service_client::QueryServiceClient as SctQueryServiceClient, EpochByHeightRequest,
};
use penumbra_sdk_sct::epoch::Epoch;
use penumbra_sdk_view::{Planner, ViewClient};
use rand_core::OsRng;

use crate::App;

async fn fetch_epoch(app: &mut App) -> anyhow::Result<Epoch> {
    let mut sct_client = SctQueryServiceClient::new(app.pd_channel().await?);
    let latest_sync_height = app.view().status().await?.full_sync_height;
    let epoch = sct_client
        .epoch_by_height(EpochByHeightRequest {
            height: latest_sync_height,
        })
        .await?
        .into_inner()
        .epoch
        .expect("epoch must be available")
        .into();
    Ok(epoch)
}

/// Vote in the current round of the liquidity tournament.
///
/// This will plan a transaction which directs all available voting power to a single asset.
#[derive(Debug, clap::Parser)]
pub struct LqtVoteCmd {
    /// The denom string for the asset being voted for.
    vote: String,
    /// If provided, make the rewards recipient a particular address instead.
    ///
    /// This can also be an integer, indicating an ephemeral address of a sub-account.
    #[clap(short, long)]
    rewards_recipient: Option<String>,
    /// The selected fee tier.
    #[clap(short, long, default_value_t)]
    fee_tier: FeeTier,
}

impl LqtVoteCmd {
    pub fn offline(&self) -> bool {
        false
    }

    fn rewards_addr(&self, app: &App) -> anyhow::Result<Address> {
        let to_parse = match &self.rewards_recipient {
            None => {
                return Ok(app
                    .config
                    .full_viewing_key
                    .ephemeral_address(OsRng, Default::default())
                    .0)
            }
            Some(x) => x,
        };
        let maybe_index: Option<u32> = to_parse.parse().ok();
        if let Some(i) = maybe_index {
            return Ok(app
                .config
                .full_viewing_key
                .ephemeral_address(OsRng, i.into())
                .0);
        }
        to_parse
            .parse()
            .map_err(|_| anyhow!("failed to parse address '{}'", to_parse))
    }

    pub async fn exec(&self, app: &mut App, gas_prices: GasPrices) -> anyhow::Result<()> {
        let vote_meta = REGISTRY
            .parse_denom(&self.vote)
            .ok_or_else(|| anyhow!("failed to parse denom: '{}'", &self.vote))?;
        let vote_denom = vote_meta.base_denom();

        let epoch = fetch_epoch(app).await?;
        let voting_notes = app.view().lqt_voting_notes(epoch.index, None).await?;

        let mut planner = Planner::new(OsRng);

        planner
            .set_gas_prices(gas_prices)
            .set_fee_tier(self.fee_tier);

        // First, tell the planner to make all the necessary votes.
        planner.lqt_vote(
            u16::try_from(epoch.index)?,
            vote_denom,
            self.rewards_addr(app)?,
            &voting_notes,
        );
        // We also want to go ahead and do the consolidation thing,
        // to reduce the number of votes we need in the next epoch.
        // To do so, we need to spend all of these notes, and produce one output per
        // delegator token.
        let mut totals: BTreeMap<asset::Id, Amount> = Default::default();
        for note in voting_notes {
            let value = note.note.value();
            planner.spend(note.note, note.position);
            *totals.entry(value.asset_id).or_insert(Amount::zero()) += value.amount;
        }
        let change_addr = app
            .config
            .full_viewing_key
            .ephemeral_address(OsRng, Default::default())
            .0;
        planner.change_address(change_addr.clone());
        for (asset_id, amount) in totals {
            planner.output(Value { asset_id, amount }, change_addr.clone());
        }

        let plan = planner.plan(app.view(), Default::default()).await?;
        app.build_and_submit_transaction(plan).await?;

        Ok(())
    }
}
