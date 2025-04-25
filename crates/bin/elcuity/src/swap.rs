use std::iter;

use anyhow::anyhow;
use clap::Args;
use decaf377::Fq;
use penumbra_sdk_asset::{
    asset::{self, REGISTRY},
    Value, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};
use penumbra_sdk_dex::swap_claim::SwapClaimPlan;
use penumbra_sdk_num::Amount;
use penumbra_sdk_transaction::gas::swap_claim_gas_cost;
use penumbra_sdk_view::Planner;
use rand_core::OsRng;

use crate::{clients::Clients, planner::submit};

#[tracing::instrument(skip(clients))]
pub async fn swap(clients: &Clients, input: Value, to: asset::Id) -> anyhow::Result<Amount> {
    tracing::info!("starting swap");
    let mut view = clients.view();
    let mut planner = Planner::new(OsRng);
    let gas_prices = view.gas_prices().await?;
    planner.set_gas_prices(gas_prices.clone());

    // We don't expect much of a drift in gas prices in a few blocks, and the fee tier
    // adjustments should be enough to cover it.
    let estimated_claim_fee = gas_prices.fee(&swap_claim_gas_cost());

    planner.swap(
        input,
        to,
        estimated_claim_fee,
        view.address_by_index(Default::default()).await?,
    )?;

    let plan = planner.plan(view.as_mut(), Default::default()).await?;

    // Hold on to the swap plaintext to be able to claim.
    let swap_plaintext = plan
        .swap_plans()
        .next()
        .expect("swap plan must be present")
        .swap_plaintext
        .clone();

    submit(clients, plan).await?;

    // Fetch the SwapRecord with the claimable swap.
    let swap_record = view
        .swap_by_commitment(swap_plaintext.swap_commitment())
        .await?;

    let pro_rata_outputs = swap_record
        .output_data
        .pro_rata_outputs((swap_plaintext.delta_1_i, swap_plaintext.delta_2_i));

    let params = view.app_params().await?;

    let mut planner = Planner::new(OsRng);
    planner.set_gas_prices(gas_prices);
    let plan = planner
        .swap_claim(SwapClaimPlan {
            swap_plaintext,
            position: swap_record.position,
            output_data: swap_record.output_data,
            epoch_duration: params.sct_params.epoch_duration,
            proof_blinding_r: Fq::rand(&mut OsRng),
            proof_blinding_s: Fq::rand(&mut OsRng),
        })
        .plan(view.as_mut(), Default::default())
        .await?;
    submit(clients, plan).await?;

    let out = if swap_record.swap.trading_pair.asset_1() == to {
        pro_rata_outputs.0
    } else {
        pro_rata_outputs.1
    };
    Ok(out)
}

#[derive(Debug, Args)]
pub struct Opt {
    #[clap(long, display_order = 100)]
    /// The starting amount to allocate for trades, specified in UM.
    ///
    /// Once a cycle of trading is completed, and the assets are swapped back into UM,
    /// the value is topped up by withdrawing additional UM from the wallet's balance
    /// to reach the declared starting amount again. Thus each cycle will drain a bit more
    /// funds from the balance, potentially draining the wallet entirely.
    starting_amount: u32,
    #[clap(long, display_order = 200)]
    /// The asset(s) to swap UM for. Can be declared multiple times.
    cycle: Vec<String>,
}

impl Opt {
    pub async fn run(self, clients: &Clients) -> anyhow::Result<()> {
        let asset_ids = self
            .cycle
            .iter()
            .map(|x| {
                REGISTRY
                    .parse_denom(x)
                    .ok_or(anyhow!("failed to parse denom '{}'", x))
                    .map(|x| x.id())
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let mut cycle = iter::once(*STAKING_TOKEN_ASSET_ID)
            .chain(asset_ids.into_iter())
            .cycle()
            .peekable();
        let starting_amount = STAKING_TOKEN_DENOM
            .default_unit()
            .value(Amount::from(self.starting_amount))
            .amount;
        let mut amount = starting_amount;
        while let Some(asset_id) = cycle.next() {
            // Top up the amount, in case we've lost value for whatever reason
            if asset_id == *STAKING_TOKEN_ASSET_ID {
                amount = amount.max(starting_amount);
            }
            let &next = cycle.peek().expect("cycle should be infinite");
            amount = swap(clients, Value { asset_id, amount }, next).await?;
        }
        Ok(())
    }
}
