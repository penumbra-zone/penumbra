use anyhow::anyhow;
use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use clap::Args;
use penumbra_sdk_asset::{
    asset::{self, REGISTRY},
    Value, STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};
use penumbra_sdk_dex::{
    lp::{
        position::{self, Position},
        Reserves,
    },
    DirectedTradingPair, SwapExecution, TradingPair,
};
use penumbra_sdk_num::{fixpoint::U128x128, Amount};
use penumbra_sdk_proto::core::component::dex::v1::{
    simulate_trade_request::{routing::Setting, Routing},
    LiquidityPositionByIdRequest, SimulateTradeRequest,
};
use rand_core::OsRng;
use tokio::time::Instant;

use crate::{clients::Clients, planner::build_and_submit};

const POSITION_CHUNK_SIZE: usize = 5;
const DEFAULT_PRICE: u8 = 1;
const DEFAULT_FEE_BPS: u16 = 500;
const ADJUST_WAIT_SECS: u64 = 120;

async fn close_all_positions(clients: &Clients, pair: TradingPair) -> anyhow::Result<()> {
    let mut view = clients.view();
    let position_ids = view
        .owned_position_ids(Some(position::State::Opened), Some(pair), None)
        .await?;
    for chunk in position_ids.chunks(POSITION_CHUNK_SIZE) {
        let tx = build_and_submit(clients, Default::default(), |mut planner| async {
            for position_id in chunk {
                planner.position_close(*position_id);
            }
            Ok(planner)
        })
        .await?;
        tracing::info!(
            tx_id = format!("{}", tx.id()),
            "closed {} positions",
            chunk.len()
        );
    }

    Ok(())
}

async fn withdraw_all_positions(clients: &Clients, pair: TradingPair) -> anyhow::Result<()> {
    let mut view = clients.view();
    let mut dex_query_service = clients.dex_query_service();
    let position_ids = view
        .owned_position_ids(Some(position::State::Closed), Some(pair), None)
        .await?;
    for chunk in position_ids.chunks(POSITION_CHUNK_SIZE) {
        let tx = build_and_submit(clients, Default::default(), |mut planner| async {
            for position_id in chunk {
                let position: Position = dex_query_service
                    .liquidity_position_by_id(LiquidityPositionByIdRequest {
                        position_id: Some((*position_id).into()),
                    })
                    .await?
                    .into_inner()
                    .data
                    .ok_or(anyhow!("expected position to be known by the Dex"))?
                    .try_into()?;

                planner.position_withdraw(*position_id, position.reserves, position.phi.pair, 0);
            }
            Ok(planner)
        })
        .await?;
        tracing::info!(
            tx_id = format!("{}", tx.id()),
            "closed {} positions",
            chunk.len()
        );
    }

    Ok(())
}

async fn get_sell_price(
    clients: &Clients,
    pair: DirectedTradingPair,
) -> anyhow::Result<Option<U128x128>> {
    let mut simulation_service = clients.dex_simulation_service();
    let resp: SwapExecution = simulation_service
        .simulate_trade(SimulateTradeRequest {
            input: Some(
                Value {
                    asset_id: pair.start,
                    amount: 1_000_000u64.into(),
                }
                .into(),
            ),
            output: Some(pair.end.into()),
            routing: Some(Routing {
                setting: Some(Setting::Default(Default::default())),
            }),
        })
        .await?
        .into_inner()
        .output
        .ok_or_else(|| anyhow::anyhow!("proto response missing swap execution"))?
        .try_into()?;
    let Some(first_trace) = resp.traces.first() else {
        return Ok(None);
    };
    if first_trace.is_empty() {
        anyhow::bail!("first trace from swap execution is empty");
    }
    let (first_value, last_value) = (
        first_trace.first().expect("should not be empty"),
        first_trace.last().expect("should not be empty"),
    );
    if first_value.asset_id != pair.start || last_value.asset_id != pair.end {
        anyhow::bail!("trace did not match directed pair {:?}", pair);
    }
    // How much output did I get per unit of input at the first price?
    Ok(U128x128::ratio(last_value.amount, first_value.amount).ok())
}

async fn spread(
    clients: &Clients,
    pair: DirectedTradingPair,
) -> anyhow::Result<Option<(U128x128, U128x128)>> {
    let Some(sell_price) = get_sell_price(clients, pair).await? else {
        return Ok(None);
    };
    let Some(other_sell_price) = get_sell_price(clients, pair.flip()).await? else {
        return Ok(None);
    };
    let buy_price = U128x128::ratio(1u64.into(), other_sell_price)?;
    Ok(Some((buy_price, sell_price)))
}

fn spread_to_price_and_fee_bps(buy: U128x128, sell: U128x128) -> (U128x128, u16) {
    // Make sure that buy >= sell.
    let (buy, sell) = if buy >= sell {
        (buy, sell)
    } else {
        (sell, buy)
    };
    let epsilon = {
        // Big endian representation, so setting the least significant bit is the smallest we can get.
        let mut bytes = [0u8; 32];
        bytes[31] = 1;
        U128x128::from_bytes(bytes)
    };
    // To try and guarantee some amount of intra-spread, squeeze the prices infinitessimally.
    let buy = (buy - epsilon).expect("buy should not be 0");
    let sell = (sell + epsilon).expect("sell should not be the maximum value");
    // x (1 + f) = b ; x (1 - f) = s =>
    // f = (b / x) - 1 ; x = (b + s) / 2
    let midpoint: U128x128 =
        ((buy + sell) / U128x128::from(2u8)).expect("division by 2 should work");
    let fee: U128x128 = ((buy / midpoint).expect("midpoint should not be 0") - U128x128::from(1u8))
        .expect("midpoint should be less than buy price");
    let fee_as_u16 = u16::min(
        5000,
        fee.apply_to_amount(&Amount::from(10_000u16))
            .ok()
            .and_then(|x| x.value().try_into().ok())
            .unwrap_or(10_000),
    );
    (midpoint, fee_as_u16)
}

/// Figure out a limit on what balances we can safely use.
async fn calculate_balance_limits(clients: &Clients) -> anyhow::Result<HashMap<asset::Id, Amount>> {
    let mut out: HashMap<_, _> = clients
        .view()
        .balances(Default::default(), None)
        .await?
        .into_iter()
        .collect();
    // Only use 90% of the staking token, to allow for paying fees.
    match out.entry(*STAKING_TOKEN_ASSET_ID) {
        Entry::Occupied(o) => {
            let v = o.into_mut();
            *v -= v.checked_mul(&10u8.into()).unwrap_or(0u8.into()) / 100u8.into();
        }
        _ => {}
    }
    Ok(out)
}

fn price_to_p_q(price: U128x128, precision: Amount) -> (Amount, Amount) {
    let mut p = price;
    let mut q = Amount::from(1u8);
    let precision_u128x128 = U128x128::from(precision);
    while !p.is_integral() {
        let Ok(new_p) = p.checked_mul(&2u8.into()) else {
            break;
        };
        let Some(new_q) = q.checked_mul(&2u8.into()) else {
            break;
        };
        if new_p > precision_u128x128 || new_q > precision {
            break;
        }
        p = new_p;
        q = new_q;
    }
    (
        Amount::from_be_bytes(
            p.round_down().to_bytes()[..16]
                .try_into()
                .expect("should be 16 bytes"),
        ),
        q,
    )
}

fn make_position(
    balance_limits: &HashMap<asset::Id, Amount>,
    pair: DirectedTradingPair,
    liquidity_target: Value,
    price: U128x128,
    fee_bps: u16,
) -> Position {
    let half_liquidity_target = liquidity_target.amount / Amount::from(2u8);
    let other_target = price
        .apply_to_amount(&half_liquidity_target)
        .expect("should be able to multiply by price");
    let (mut start_target, mut end_target) = if liquidity_target.asset_id == pair.end {
        (other_target, half_liquidity_target)
    } else if liquidity_target.asset_id == pair.start {
        (half_liquidity_target, other_target)
    } else {
        panic!("you should make sure that the pair and the liquidity target match");
    };
    if let Some(limit) = balance_limits.get(&pair.start) {
        start_target = start_target.min(*limit);
    }
    if let Some(limit) = balance_limits.get(&pair.end) {
        end_target = end_target.min(*limit);
    }
    let (p, q) = price_to_p_q(price, (1u64 << 60).into());
    Position::new(
        OsRng,
        pair,
        fee_bps.into(),
        p,
        q,
        Reserves {
            r1: start_target,
            r2: end_target,
        },
    )
}

async fn create_position_at_current_spread(
    clients: &Clients,
    pair: DirectedTradingPair,
    liquidity_target: Value,
    default_price: U128x128,
) -> anyhow::Result<()> {
    let (price, fee_bps) = match spread(clients, pair).await? {
        Some((buy, sell)) => spread_to_price_and_fee_bps(buy, sell),
        None => (default_price, DEFAULT_FEE_BPS),
    };
    let balance_limits = calculate_balance_limits(clients).await?;
    let position = make_position(&balance_limits, pair, liquidity_target, price, fee_bps);
    tracing::info!(?position, "creating position");
    let tx = build_and_submit(clients, Default::default(), |mut planner| async {
        planner.position_open(position);
        Ok(planner)
    })
    .await?;
    tracing::info!(tx_id = format!("{}", tx.id()), "created position");
    Ok(())
}

#[tracing::instrument(skip(clients))]
async fn adjust_liquidity_provision(
    clients: &Clients,
    other_asset: asset::Id,
    liquidity_target: Amount,
    default_price: U128x128,
) -> anyhow::Result<()> {
    let pair = DirectedTradingPair::new(other_asset, *STAKING_TOKEN_ASSET_ID);
    let target = Value {
        asset_id: *STAKING_TOKEN_ASSET_ID,
        amount: liquidity_target,
    };
    close_all_positions(clients, pair.to_canonical()).await?;
    withdraw_all_positions(clients, pair.to_canonical()).await?;
    create_position_at_current_spread(clients, pair, target, default_price).await?;
    Ok(())
}

#[derive(Debug, Args)]
pub struct Opt {
    /// The denom to provide liquidity for, relative to the staking token.
    ///
    /// Must be specified as an IBC transfer asset, e.g. `transfer/channel-1/uusdc`.
    #[clap(long = "for", display_order = 100)]
    denom: String,
    /// The amount of liquidity to provide, in terms of the staking token.
    ///
    /// For instance, `elcuity lp --for transfer/channel-1/uusdc --liquidity-target 100`
    /// would provision 1) `50penumbra` and 2) `50penumbra` worth of `transfer/channel-1/uusdc`,
    /// resolving the price for the latter on the DEX, or using the `--default-price` arg if set.
    #[clap(long, display_order = 200)]
    liquidity_target: u32,
    /// If provided, a price to use instead of 1 as the default.
    ///
    /// The price of the IBC transfer asset will be looked up on the DEX. If no price is found on
    /// the DEX, then the default value of `1` will be used, meaning `
    #[clap(long, display_order = 300)]
    default_price: Option<f64>,
}

impl Opt {
    pub async fn run(self, clients: &Clients) -> anyhow::Result<()> {
        let other_asset = REGISTRY
            .parse_denom(&self.denom)
            .ok_or(anyhow!("failed to parse denom '{}'", &self.denom))?
            .id();
        let liquidity_target = STAKING_TOKEN_DENOM
            .default_unit()
            .value(Amount::from(self.liquidity_target))
            .amount;
        let default_price = self
            .default_price
            .map(|x| U128x128::try_from(x))
            .transpose()?
            .unwrap_or(U128x128::from(DEFAULT_PRICE));
        loop {
            let start = Instant::now();
            if let Err(e) =
                adjust_liquidity_provision(clients, other_asset, liquidity_target, default_price)
                    .await
            {
                tracing::error!(
                    error = format!("{}", e),
                    "error adjusting liquidity provision"
                );
            }
            tokio::time::sleep_until(start + Duration::from_secs(ADJUST_WAIT_SECS)).await;
        }
    }
}
