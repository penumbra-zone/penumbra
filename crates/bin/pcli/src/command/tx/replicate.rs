use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use dialoguer::Confirm;
use rand_core::OsRng;

use penumbra_asset::Value;
use penumbra_dex::{lp::position::Position, DirectedUnitPair};
use penumbra_keys::keys::AddressIndex;
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{
    core::component::dex::v1alpha1::{
        query_service_client::QueryServiceClient as DexQueryServiceClient, SpreadRequest,
    },
    view::v1alpha1::GasPricesRequest,
};
use penumbra_view::{Planner, ViewClient};

use crate::dex_utils;
use crate::dex_utils::replicate::debug;
use crate::{warning, App};

/// Queries the chain for a transaction by hash.
#[derive(Debug, clap::Subcommand)]
pub enum ReplicateCmd {
    #[clap(visible_alias = "xyk")]
    ConstantProduct(ConstantProduct),
}

impl ReplicateCmd {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        match self {
            ReplicateCmd::ConstantProduct(xyk_cmd) => xyk_cmd.exec(app).await?,
        };
        Ok(())
    }

    pub fn offline(&self) -> bool {
        match self {
            ReplicateCmd::ConstantProduct(_) => false,
        }
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct ConstantProduct {
    pub pair: DirectedUnitPair,
    pub input: Value,

    #[clap(short, long)]
    pub current_price: Option<f64>,

    #[clap(short, long, default_value_t = 0u32)]
    pub fee_bps: u32,
    /// `--yes` means all prompt interaction are skipped and agreed.
    #[clap(short, long)]
    pub yes: bool,

    #[clap(short, long, hide(true))]
    pub debug_file: Option<PathBuf>,
    #[clap(long, default_value = "0", hide(true))]
    pub source: u32,
}

impl ConstantProduct {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        self.validate()?;
        let pair = self.pair.clone();
        let current_price = match self.current_price {
            Some(user_supplied_price) => user_supplied_price,
            None => self.get_spread(app).await?,
        };

        let positions = dex_utils::replicate::xyk::replicate(
            &pair,
            &self.input,
            current_price.try_into()?,
            self.fee_bps,
        )?;

        let (amount_start, amount_end) =
            positions
                .iter()
                .fold((Amount::zero(), Amount::zero()), |acc, pos| {
                    (
                        acc.0
                            + pos
                                .reserves_for(pair.start.id())
                                .expect("start is part of position"),
                        acc.1
                            + pos
                                .reserves_for(pair.end.id())
                                .expect("end is part of position"),
                    )
                });
        let amount_start = pair.start.format_value(amount_start);
        let amount_end = pair.end.format_value(amount_end);

        warning::rmm();

        if !self.yes
            && !Confirm::new()
                .with_prompt("In the solemn voice of Mandos, he who sets the fates of all, you hear a question,\nechoing like a whisper through the Halls of Waiting:\n\"Do you, in your heart of hearts, truly wish to proceed?\"")
                .interact()?
        {
            return Ok(());
        }
        println!("\nso it shall be...\n\n");
        println!(
            "#################################################################################"
        );
        println!(
            "########################### LIQUIDITY SUMMARY ###################################"
        );
        println!(
            "#################################################################################"
        );
        println!("\nYou want to provide liquidity on the pair {}", pair);
        println!("You will need:",);
        println!(" -> {amount_start}{}", pair.start);
        println!(" -> {amount_end}{}", pair.end);
        // TODO(erwan): would be nice to print current balance?

        println!("You will create the following pools:");
        let asset_cache = app.view().assets().await?;
        println!(
            "{}",
            crate::command::utils::render_positions(&asset_cache, &positions),
        );

        if let Some(debug_file) = &self.debug_file {
            Self::write_debug_data(
                debug_file.clone(),
                self.pair.clone(),
                self.input.clone(),
                current_price,
                positions.clone(),
            )?;
            return Ok(());
        }

        if !self.yes
            && !Confirm::new()
                .with_prompt("Do you want to open those liquidity positions on-chain?")
                .interact()?
        {
            return Ok(());
        }

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

        let mut planner = Planner::new(OsRng);
        planner.set_gas_prices(gas_prices);
        positions.iter().for_each(|position| {
            planner.position_open(position.clone());
        });

        let plan = planner
            .plan(
                app.view
                    .as_mut()
                    .context("view service must be initialized")?,
                AddressIndex::new(self.source),
            )
            .await?;
        let tx_id = app.build_and_submit_transaction(plan).await?;
        println!("posted with transaction id: {tx_id}");

        Ok(())
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.input.asset_id != self.pair.start.id() && self.input.asset_id != self.pair.end.id()
        {
            anyhow::bail!("you must supply liquidity with an asset that's part of the market")
        } else if self.input.amount == 0u64.into() {
            anyhow::bail!("the quantity of liquidity supplied must be non-zero.",)
        } else if self.fee_bps > 5000 {
            anyhow::bail!("the maximum fee is 5000bps (50%)")
        } else if self.current_price.is_some()
            && self.current_price.expect("current price is Some") <= 0.0
        {
            anyhow::bail!("the supplied current price must be positive")
        } else {
            Ok(())
        }
    }

    async fn get_spread(&self, app: &mut App) -> Result<f64> {
        let mut client = DexQueryServiceClient::new(app.pd_channel().await?);
        let spread_data = client
            .spread(SpreadRequest {
                chain_id: "".to_string(),
                trading_pair: Some(self.pair.into_directed_trading_pair().to_canonical().into()),
            })
            .await?
            .into_inner();

        tracing::debug!(
            ?spread_data,
            pair = self.pair.to_string(),
            "fetched spread for pair"
        );

        if spread_data.best_1_to_2_position.is_none() || spread_data.best_2_to_1_position.is_none()
        {
            bail!("couldn't find a market price for the specified assets, you can manually specify a price using --current-price <price>")
        }

        if self.input.asset_id == self.pair.start.id() {
            Ok(spread_data.approx_effective_price_1_to_2)
        } else if self.input.asset_id == self.pair.end.id() {
            Ok(spread_data.approx_effective_price_2_to_1)
        } else {
            bail!("the supplied liquidity must be on the pair")
        }
    }

    pub(crate) fn write_debug_data(
        file: PathBuf,
        pair: DirectedUnitPair,
        input: Value,
        current_price: f64,
        positions: Vec<Position>,
    ) -> anyhow::Result<()> {
        // Ad-hoc denom scaling for debug data:
        let alphas = dex_utils::replicate::xyk::sample_prices(
            current_price,
            dex_utils::replicate::xyk::NUM_POOLS_PRECISION,
        );

        alphas
            .iter()
            .enumerate()
            .for_each(|(i, alpha)| tracing::debug!(i, alpha, "sampled tick"));

        let r1: f64;

        {
            let raw_r1 = input.amount.value();
            let denom_unit = pair.start.unit_amount().value();
            let fp_r1 = U128x128::ratio(raw_r1, denom_unit).expect("denom unit is not 0");
            r1 = fp_r1.into();
        }

        let r2 = r1 * current_price;
        let total_k = r1 * r2;
        println!("Entry R1: {r1}");
        println!("Entry R2: {r2}");
        println!("total K: {total_k}");

        let debug_positions: Vec<debug::PayoffPositionEntry> = positions
            .iter()
            .zip(alphas)
            .enumerate()
            .map(|(idx, (pos, alpha))| {
                let payoff_entry = debug::PayoffPosition::from_position(pair.clone(), pos.clone());
                debug::PayoffPositionEntry {
                    payoff: payoff_entry,
                    current_price,
                    index: idx,
                    pair: pair.clone(),
                    alpha,
                    total_k,
                }
            })
            .collect();

        let mut fd = std::fs::File::create(&file).map_err(|e| {
            anyhow!(
                "fs error opening debug file {}: {}",
                file.to_string_lossy(),
                e
            )
        })?;

        let json_data = serde_json::to_string(&debug_positions)
            .map_err(|e| anyhow!("error serializing PayoffPositionEntry: {}", e))?;

        fd.write_all(json_data.as_bytes())
            .map_err(|e| anyhow!("error writing {}: {}", file.to_string_lossy(), e))?;
        Ok(())
    }
}
