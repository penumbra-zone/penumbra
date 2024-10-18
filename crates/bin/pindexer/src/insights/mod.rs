use std::{collections::BTreeMap, iter};

use cometindex::{async_trait, AppView, ContextualizedEvent, PgTransaction};
use penumbra_app::genesis::Content;
use penumbra_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_dex::{
    event::{EventArbExecution, EventPositionClose, EventPositionExecution, EventPositionOpen},
    lp::position::{self, Position},
    TradingPair,
};
use penumbra_fee::event::EventBlockFees;
use penumbra_funding::event::EventFundingStreamReward;
use penumbra_num::Amount;
use penumbra_proto::{event::EventDomainType, DomainType, Name};
use penumbra_shielded_pool::event::{
    EventInboundFungibleTokenTransfer, EventOutboundFungibleTokenRefund,
    EventOutboundFungibleTokenTransfer,
};
use penumbra_stake::{
    event::{EventDelegate, EventRateDataChange, EventUndelegate},
    validator::Validator,
    IdentityKey,
};
use sqlx::PgPool;

use crate::parsing::parse_content;

#[derive(Debug, Clone, Copy)]
struct ValidatorSupply {
    um: u64,
    rate_bps2: u64,
}

async fn modify_validator_supply(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    ik: IdentityKey,
    f: Box<dyn FnOnce(ValidatorSupply) -> anyhow::Result<ValidatorSupply> + Send + 'static>,
) -> anyhow::Result<i64> {
    let ik_text = ik.to_string();
    let supply = {
        let row: Option<(i64, i64)> = sqlx::query_as("
            SELECT um, rate_bps2 FROM _insights_validators WHERE validator_id = $1 ORDER BY height DESC LIMIT 1
        ").bind(&ik_text).fetch_optional(dbtx.as_mut()).await?;
        let row = row.unwrap_or((0i64, 1_0000_0000i64));
        ValidatorSupply {
            um: u64::try_from(row.0)?,
            rate_bps2: u64::try_from(row.1)?,
        }
    };
    let new_supply = f(supply)?;
    sqlx::query(
        r#"
        INSERT INTO _insights_validators 
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (validator_id, height) DO UPDATE SET
            um = excluded.um,
            rate_bps2 = excluded.rate_bps2
    "#,
    )
    .bind(&ik_text)
    .bind(i64::try_from(height)?)
    .bind(i64::try_from(new_supply.um)?)
    .bind(i64::try_from(new_supply.rate_bps2)?)
    .execute(dbtx.as_mut())
    .await?;
    Ok(i64::try_from(new_supply.um)? - i64::try_from(supply.um)?)
}

#[derive(Default, Debug, Clone, Copy)]
struct Supply {
    total: u64,
    staked: u64,
}

async fn modify_supply(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    price_numeraire: Option<asset::Id>,
    min_reserves: f64,
    f: Box<dyn FnOnce(Supply) -> anyhow::Result<Supply> + Send + 'static>,
) -> anyhow::Result<()> {
    let supply: Supply = {
        let row: Option<(i64, i64)> = sqlx::query_as(
            "SELECT total, staked FROM insights_supply ORDER BY HEIGHT DESC LIMIT 1",
        )
        .fetch_optional(dbtx.as_mut())
        .await?;
        row.map(|(total, staked)| {
            anyhow::Result::<_>::Ok(Supply {
                total: total.try_into()?,
                staked: staked.try_into()?,
            })
        })
        .transpose()?
        .unwrap_or_default()
    };
    let supply = f(supply)?;
    sqlx::query(
        r#"
        INSERT INTO 
            insights_supply(height, total, staked, price, price_numeraire_asset_id) 
            VALUES ($1, $2, $3, (SELECT MAX(price) FROM _insights_price_list WHERE reserves >= $5), $4)
        ON CONFLICT (height) DO UPDATE SET
        total = excluded.total,
        staked = excluded.staked,
        price = excluded.price,
        price_numeraire_asset_id = excluded.price_numeraire_asset_id
    "#,
    )
    .bind(i64::try_from(height)?)
    .bind(i64::try_from(supply.total)?)
    .bind(i64::try_from(supply.staked)?)
    .bind(price_numeraire.map(|x| x.to_bytes()))
    .bind(min_reserves)
    .execute(dbtx.as_mut())
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DepositorExisted {
    Yes,
    No,
}

async fn register_depositor(
    dbtx: &mut PgTransaction<'_>,
    asset_id: asset::Id,
    address: &str,
) -> anyhow::Result<DepositorExisted> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM _insights_shielded_pool_depositors WHERE asset_id = $1 AND address = $2)",
    )
    .bind(asset_id.to_bytes())
    .bind(address)
    .fetch_one(dbtx.as_mut())
    .await?;
    if exists {
        return Ok(DepositorExisted::Yes);
    }
    sqlx::query("INSERT INTO _insights_shielded_pool_depositors VALUES ($1, $2)")
        .bind(asset_id.to_bytes())
        .bind(address)
        .execute(dbtx.as_mut())
        .await?;
    Ok(DepositorExisted::No)
}

async fn asset_flow(
    dbtx: &mut PgTransaction<'_>,
    asset_id: asset::Id,
    height: u64,
    flow: i128,
    depositor_existed: DepositorExisted,
) -> anyhow::Result<()> {
    let asset_pool: Option<(String, String, i32)> = sqlx::query_as("SELECT total_value, current_value, unique_depositors FROM insights_shielded_pool WHERE asset_id = $1 ORDER BY height DESC LIMIT 1").bind(asset_id.to_bytes()).fetch_optional(dbtx.as_mut()).await?;
    let mut asset_pool = asset_pool
        .map(|(t, c, u)| {
            anyhow::Result::<(i128, i128, i32)>::Ok((
                i128::from_str_radix(&t, 10)?,
                i128::from_str_radix(&c, 10)?,
                u,
            ))
        })
        .transpose()?
        .unwrap_or((0i128, 0i128, 0i32));
    asset_pool.0 += flow.abs();
    asset_pool.1 += flow;
    asset_pool.2 += match depositor_existed {
        DepositorExisted::Yes => 0,
        DepositorExisted::No => 1,
    };
    sqlx::query(
        r#"
        INSERT INTO insights_shielded_pool
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (asset_id, height) DO UPDATE SET
        total_value = excluded.total_value,
        current_value = excluded.current_value,
        unique_depositors = excluded.unique_depositors
    "#,
    )
    .bind(asset_id.to_bytes())
    .bind(i64::try_from(height)?)
    .bind(asset_pool.0.to_string())
    .bind(asset_pool.1.to_string())
    .bind(asset_pool.2)
    .execute(dbtx.as_mut())
    .await?;
    Ok(())
}

#[derive(Debug)]
pub struct Component {
    price_numeraire: Option<asset::Id>,
    min_reserves: f64,
}

impl Component {
    /// This component depends on a reference asset for the total supply price.
    pub fn new(price_numeraire: Option<asset::Id>, min_reserves: f64) -> Self {
        Self {
            price_numeraire,
            min_reserves,
        }
    }

    async fn register_position(
        &self,
        dbtx: &mut PgTransaction<'_>,
        position: &Position,
    ) -> anyhow::Result<bool> {
        let price_numeraire = match self.price_numeraire {
            None => return Ok(false),
            Some(x) => x,
        };
        let pair = position.phi.pair;
        let (fee, p, q, r): (u32, Amount, Amount, Amount) =
            if pair.asset_1() == *STAKING_TOKEN_ASSET_ID && pair.asset_2() == price_numeraire {
                let c = &position.phi.component;
                (c.fee, c.p, c.q, position.reserves_2().amount)
            } else if pair.asset_2() == *STAKING_TOKEN_ASSET_ID && pair.asset_1() == price_numeraire
            {
                let c = &position.phi.component;
                (c.fee, c.q, c.p, position.reserves_1().amount)
            } else {
                return Ok(false);
            };

        let price =
            (1.0 - ((fee as f64) / 10000.0)) * (u128::from(p) as f64) / (u128::from(q) as f64);
        let id = position.id();

        sqlx::query("INSERT INTO _insights_price_list VALUES ($1, $2, $3)")
            .bind(id.0.as_slice())
            .bind(price)
            .bind(r.value() as f64)
            .execute(dbtx.as_mut())
            .await?;
        Ok(true)
    }

    async fn update_position(
        &self,
        dbtx: &mut PgTransaction<'_>,
        id: position::Id,
        reserves: Amount,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE _insights_price_list SET reserves = $2 WHERE position_id = $1")
            .bind(id.0.as_slice())
            .bind(reserves.value() as f64)
            .execute(dbtx.as_mut())
            .await?;
        Ok(())
    }
}

/// Add the initial native token supply.
async fn add_genesis_native_token_allocation_supply<'a>(
    dbtx: &mut PgTransaction<'a>,
    content: &Content,
) -> anyhow::Result<()> {
    fn content_mints(content: &Content) -> BTreeMap<asset::Id, Amount> {
        let community_pool_mint = iter::once((
            *STAKING_TOKEN_ASSET_ID,
            content.community_pool_content.initial_balance.amount,
        ));
        let allocation_mints = content
            .shielded_pool_content
            .allocations
            .iter()
            .map(|allocation| {
                let value = allocation.value();
                (value.asset_id, value.amount)
            });

        let mut out = BTreeMap::new();
        for (id, amount) in community_pool_mint.chain(allocation_mints) {
            out.entry(id).and_modify(|x| *x += amount).or_insert(amount);
        }
        out
    }

    let mints = content_mints(content);

    let unstaked = u64::try_from(
        mints
            .get(&*STAKING_TOKEN_ASSET_ID)
            .copied()
            .unwrap_or_default()
            .value(),
    )?;

    let mut staked = 0u64;
    // at genesis, assume a 1:1 ratio between delegation amount and native token amount.
    for val in &content.stake_content.validators {
        let val = Validator::try_from(val.clone())?;
        let delegation_amount: u64 = mints
            .get(&val.token().id())
            .cloned()
            .unwrap_or_default()
            .value()
            .try_into()?;
        staked += delegation_amount;
        modify_validator_supply(
            dbtx,
            0,
            val.identity_key,
            Box::new(move |_| {
                Ok(ValidatorSupply {
                    um: delegation_amount,
                    rate_bps2: 1_0000_0000,
                })
            }),
        )
        .await?;
    }

    modify_supply(
        dbtx,
        0,
        None,
        0.0,
        Box::new(move |_| {
            Ok(Supply {
                total: unstaked + staked,
                staked,
            })
        }),
    )
    .await?;

    Ok(())
}
#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        for statement in include_str!("schema.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }

        // decode the initial supply from the genesis
        // initial app state is not recomputed from events, because events are not emitted in init_chain.
        // instead, the indexer directly parses the genesis.
        add_genesis_native_token_allocation_supply(dbtx, &parse_content(app_state.clone())?)
            .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        [
            <EventUndelegate as DomainType>::Proto::full_name(),
            <EventDelegate as DomainType>::Proto::full_name(),
            <EventRateDataChange as DomainType>::Proto::full_name(),
            <EventBlockFees as DomainType>::Proto::full_name(),
            <EventArbExecution as DomainType>::Proto::full_name(),
            <EventPositionOpen as DomainType>::Proto::full_name(),
            <EventPositionExecution as DomainType>::Proto::full_name(),
            <EventPositionClose as DomainType>::Proto::full_name(),
            <EventInboundFungibleTokenTransfer as DomainType>::Proto::full_name(),
            <EventOutboundFungibleTokenRefund as DomainType>::Proto::full_name(),
            <EventOutboundFungibleTokenTransfer as DomainType>::Proto::full_name(),
            <EventFundingStreamReward as DomainType>::Proto::full_name(),
        ]
        .into_iter()
        .any(|x| type_str == x)
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        let height = event.block_height;
        if let Ok(e) = EventUndelegate::try_from_event(&event.event) {
            let delta = modify_validator_supply(
                dbtx,
                height,
                e.identity_key,
                Box::new(move |supply| {
                    Ok(ValidatorSupply {
                        um: supply.um + u64::try_from(e.amount.value()).expect(""),
                        ..supply
                    })
                }),
            )
            .await?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                self.min_reserves,
                Box::new(move |supply| {
                    // The amount staked has changed, but no inflation has happened.
                    Ok(Supply {
                        staked: u64::try_from(i64::try_from(supply.staked)? + delta)?,
                        ..supply
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventDelegate::try_from_event(&event.event) {
            let delta = modify_validator_supply(
                dbtx,
                height,
                e.identity_key,
                Box::new(move |supply| {
                    Ok(ValidatorSupply {
                        um: supply.um + u64::try_from(e.amount.value()).expect(""),
                        ..supply
                    })
                }),
            )
            .await?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                self.min_reserves,
                Box::new(move |supply| {
                    Ok(Supply {
                        staked: u64::try_from(i64::try_from(supply.staked)? + delta)?,
                        ..supply
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventRateDataChange::try_from_event(&event.event) {
            let delta = modify_validator_supply(
                dbtx,
                height,
                e.identity_key,
                Box::new(move |supply| {
                    // del_um <- um / old_exchange_rate
                    // um <- del_um * new_exchange_rate
                    // so
                    // um <- um * (new_exchange_rate / old_exchange_rate)
                    // and the bps cancel out.
                    let um = (u128::from(supply.um) * e.rate_data.validator_exchange_rate.value())
                        .checked_div(supply.rate_bps2.into())
                        .unwrap_or(0u128)
                        .try_into()?;
                    Ok(ValidatorSupply {
                        um,
                        rate_bps2: u64::try_from(e.rate_data.validator_exchange_rate.value())?,
                    })
                }),
            )
            .await?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                self.min_reserves,
                Box::new(move |supply| {
                    // Value has been created or destroyed!
                    Ok(Supply {
                        total: u64::try_from(i64::try_from(supply.total)? + delta)?,
                        staked: u64::try_from(i64::try_from(supply.staked)? + delta)?,
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventBlockFees::try_from_event(&event.event) {
            let value = e.swapped_fee_total.value();
            if value.asset_id == *STAKING_TOKEN_ASSET_ID {
                let amount = u64::try_from(value.amount.value())?;
                // We consider the tip to be destroyed too, matching the current logic
                // DRAGON: if this changes, this code should use the base fee only.
                modify_supply(
                    dbtx,
                    height,
                    self.price_numeraire,
                    self.min_reserves,
                    Box::new(move |supply| {
                        Ok(Supply {
                            total: supply.total + amount,
                            ..supply
                        })
                    }),
                )
                .await?;
            }
        } else if let Ok(e) = EventArbExecution::try_from_event(&event.event) {
            let input = e.swap_execution.input;
            let output = e.swap_execution.output;
            if input.asset_id == *STAKING_TOKEN_ASSET_ID
                && output.asset_id == *STAKING_TOKEN_ASSET_ID
            {
                let profit = u64::try_from((output.amount - input.amount).value())?;
                modify_supply(
                    dbtx,
                    height,
                    self.price_numeraire,
                    self.min_reserves,
                    Box::new(move |supply| {
                        Ok(Supply {
                            total: supply.total + profit,
                            ..supply
                        })
                    }),
                )
                .await?;
            }
        } else if let Ok(e) = EventFundingStreamReward::try_from_event(&event.event) {
            let amount = u64::try_from(e.reward_amount.value())?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                self.min_reserves,
                Box::new(move |supply| {
                    Ok(Supply {
                        total: supply.total + amount,
                        ..supply
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventPositionOpen::try_from_event(&event.event) {
            let price_affected = self.register_position(dbtx, &e.position).await?;
            if price_affected {
                // This will update the price, if need be.
                modify_supply(
                    dbtx,
                    height,
                    self.price_numeraire,
                    self.min_reserves,
                    Box::new(|supply| Ok(Supply { ..supply })),
                )
                .await?;
            }
        } else if let Ok(e) = EventPositionExecution::try_from_event(&event.event) {
            if let Some(price_numeraire) = self.price_numeraire {
                if e.trading_pair == TradingPair::new(*STAKING_TOKEN_ASSET_ID, price_numeraire) {
                    let reserves = if e.trading_pair.asset_1() == price_numeraire {
                        e.reserves_1
                    } else {
                        e.reserves_2
                    };
                    self.update_position(dbtx, e.position_id, reserves).await?;
                }
            }
        } else if let Ok(e) = EventPositionClose::try_from_event(&event.event) {
            self.update_position(dbtx, e.position_id, 0u128.into())
                .await?;
            // This will update the price, if need be.
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                self.min_reserves,
                Box::new(|supply| Ok(Supply { ..supply })),
            )
            .await?;
        } else if let Ok(e) = EventInboundFungibleTokenTransfer::try_from_event(&event.event) {
            if e.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                let existed = register_depositor(dbtx, e.value.asset_id, &e.sender).await?;
                let flow = i128::try_from(e.value.amount.value())?;
                asset_flow(dbtx, e.value.asset_id, height, flow, existed).await?;
            }
        } else if let Ok(e) = EventOutboundFungibleTokenTransfer::try_from_event(&event.event) {
            if e.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                let flow = i128::try_from(e.value.amount.value())?;
                // For outbound transfers, never increment unique count
                asset_flow(dbtx, e.value.asset_id, height, -flow, DepositorExisted::No).await?;
            }
        } else if let Ok(e) = EventOutboundFungibleTokenRefund::try_from_event(&event.event) {
            if e.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                let flow = i128::try_from(e.value.amount.value())?;
                // For outbound transfers, never increment unique count.
                asset_flow(dbtx, e.value.asset_id, height, flow, DepositorExisted::No).await?;
            }
        }
        tracing::debug!(?event, "unrecognized event");
        Ok(())
    }
}
