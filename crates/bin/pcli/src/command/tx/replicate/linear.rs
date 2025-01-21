use anyhow::Context;
use dialoguer::Confirm;
use rand_core::{CryptoRngCore, OsRng};

use penumbra_sdk_asset::Value;
use penumbra_sdk_dex::{
    lp::{position::Position, Reserves},
    DirectedUnitPair,
};
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::view::v1::GasPricesRequest;
use penumbra_sdk_view::{Planner, ViewClient};

use crate::App;

#[derive(Debug, Clone, clap::Args)]
pub struct Linear {
    /// The pair to provide liquidity for.
    pub pair: DirectedUnitPair,

    /// The target amount of liquidity (in asset 2) to provide.
    ///
    /// Note that the actual amount of liquidity provided will be a mix of
    /// asset 1 and asset 2, depending on the current price.
    pub input: Value,

    /// The lower bound of the price range.
    ///
    /// Prices are the amount of asset 2 required to purchase 1 unit of asset 1.
    #[clap(short, long, display_order = 100)]
    pub lower_price: f64,
    /// The upper bound of the price range.
    ///
    /// Prices are the amount of asset 2 required to purchase 1 unit of asset 1.
    #[clap(short, long, display_order = 101)]
    pub upper_price: f64,

    /// The percentage fee to apply to each trade, expressed in basis points.
    #[clap(short, long, default_value_t = 50u32, display_order = 200)]
    pub fee_bps: u32,

    /// The number of positions to create.
    #[clap(short, long, default_value_t = 16, display_order = 300)]
    pub num_positions: u32,

    /// The current price. If not provided, the current price is fetched from
    /// the chain.
    ///
    /// This is used to determine which positions should be funded with asset 1
    /// and which positions should be funded with asset 2.
    #[clap(short, long, display_order = 400)]
    pub current_price: Option<f64>,

    /// Closes positions on fill, for executing trades on the maker side.
    ///
    /// Not recommended for liquidity provision
    #[clap(long, default_value_t = false, display_order = 500)]
    pub close_on_fill: bool,

    /// `--yes` means all prompt interaction are skipped and agreed.
    #[clap(short, long, display_order = 501)]
    pub yes: bool,

    /// The account to use to fund the LPs and store the LP tokens.
    #[clap(long, default_value = "0", display_order = 503)]
    pub source: u32,
}

impl Linear {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        self.validate()?;

        let pair = self.pair.clone();

        tracing::debug!(start = ?pair.start.base());
        tracing::debug!(end = ?pair.end.base());

        let mut asset_cache = app.view().assets().await?;
        if !asset_cache.contains_key(&pair.start.id()) {
            asset_cache.extend(std::iter::once(pair.start.base()));
        }
        if !asset_cache.contains_key(&pair.end.id()) {
            asset_cache.extend(std::iter::once(pair.end.base()));
        }

        let current_price =
            super::process_price_or_fetch_spread(app, self.current_price, self.pair.clone())
                .await?;

        tracing::debug!(?self);
        tracing::debug!(?current_price);

        let positions = self.build_positions(OsRng, current_price, self.input);

        let (amount_start, amount_end) =
            positions
                .iter()
                .fold((Amount::zero(), Amount::zero()), |acc, pos| {
                    tracing::debug!(?pos);
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

        println!("You will create the following positions:");
        println!(
            "{}",
            crate::command::utils::render_positions(&asset_cache, &positions),
        );

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

    fn build_positions<R: CryptoRngCore>(
        &self,
        mut rng: R,
        current_price: f64,
        input: Value,
    ) -> Vec<Position> {
        // The step width is num_positions-1 because it's between the endpoints
        // |---|---|---|---|
        // 0   1   2   3   4
        //   0   1   2   3
        let step_width = (self.upper_price - self.lower_price) / (self.num_positions - 1) as f64;

        // We are treating asset 2 as the numeraire and want to have an even spread
        // of asset 2 value across all positions.
        let total_input = input.amount.value() as f64;
        let asset_2_per_position = total_input / self.num_positions as f64;

        tracing::debug!(
            ?current_price,
            ?step_width,
            ?total_input,
            ?asset_2_per_position
        );

        let mut positions = vec![];

        let dtp = self.pair.into_directed_trading_pair();

        for i in 0..self.num_positions {
            let position_price = self.lower_price + step_width * i as f64;

            // Cross-multiply exponents and prices for trading function coefficients
            //
            // We want to write
            // p = EndUnit * price
            // q = StartUnit
            // However, if EndUnit is too small, it might not round correctly after multiplying by price
            // To handle this, conditionally apply a scaling factor if the EndUnit amount is too small.
            let scale = if self.pair.end.unit_amount().value() < 1_000_000 {
                1_000_000
            } else {
                1
            };

            let p = Amount::from(
                ((self.pair.end.unit_amount().value() * scale) as f64 * position_price) as u128,
            );
            let q = self.pair.start.unit_amount() * Amount::from(scale);

            // Compute reserves
            let reserves = if position_price < current_price {
                // If the position's price is _less_ than the current price, fund it with asset 2
                // so the position isn't immediately arbitraged.
                Reserves {
                    r1: Amount::zero(),
                    r2: Amount::from(asset_2_per_position as u128),
                }
            } else {
                // If the position's price is _greater_ than the current price, fund it with
                // an equivalent amount of asset 1 as the target per-position amount of asset 2.
                let asset_1 = asset_2_per_position / position_price;
                Reserves {
                    r1: Amount::from(asset_1 as u128),
                    r2: Amount::zero(),
                }
            };

            let position = Position::new(&mut rng, dtp, self.fee_bps, p, q, reserves);

            positions.push(position);
        }

        positions
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.input.asset_id != self.pair.end.id() {
            anyhow::bail!("liquidity target is specified in terms of asset 2 but provided input is for a different asset")
        } else if self.input.amount == 0u64.into() {
            anyhow::bail!("the quantity of liquidity supplied must be non-zero.",)
        } else if self.fee_bps > 5000 {
            anyhow::bail!("the maximum fee is 5000bps (50%)")
        } else if self.current_price.is_some()
            && self.current_price.expect("current price is Some") <= 0.0
        {
            anyhow::bail!("the supplied current price must be positive")
        } else if self.lower_price >= self.upper_price {
            anyhow::bail!("the lower price must be less than the upper price")
        } else if self.num_positions <= 2 {
            anyhow::bail!("the number of positions must be greater than 2")
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    use super::*;

    #[test]
    fn sanity_check_penumbra_sdk_gm_example() {
        let params = Linear {
            pair: "penumbra:gm".parse().unwrap(),
            input: "1000gm".parse().unwrap(),
            lower_price: 1.8,
            upper_price: 2.2,
            fee_bps: 50,
            num_positions: 5,
            current_price: Some(2.05),
            close_on_fill: false,
            yes: false,
            source: 0,
        };

        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        let positions = params.build_positions(
            &mut rng,
            params.current_price.unwrap(),
            params.input.clone(),
        );

        for position in &positions {
            dbg!(position);
        }

        let asset_cache = penumbra_sdk_asset::asset::Cache::with_known_assets();

        dbg!(&params);
        println!(
            "{}",
            crate::command::utils::render_positions(&asset_cache, &positions),
        );

        for position in &positions {
            let id = position.id();
            let buy = position.interpret_as_buy().unwrap();
            let sell = position.interpret_as_sell().unwrap();
            println!("{}: BUY  {}", id, buy.format(&asset_cache).unwrap());
            println!("{}: SELL {}", id, sell.format(&asset_cache).unwrap());
        }

        let um_id = params.pair.start.id();
        let gm_id = params.pair.end.id();

        assert_eq!(positions.len(), 5);
        // These should be all GM
        assert_eq!(positions[0].reserves_for(um_id).unwrap(), 0u64.into());
        assert_eq!(positions[1].reserves_for(um_id).unwrap(), 0u64.into());
        assert_eq!(positions[2].reserves_for(um_id).unwrap(), 0u64.into());
        // These should be all UM
        assert_eq!(positions[3].reserves_for(gm_id).unwrap(), 0u64.into());
        assert_eq!(positions[4].reserves_for(gm_id).unwrap(), 0u64.into());
    }
}
