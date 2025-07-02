use anyhow::{anyhow, Context};
use ethnum::I256;
use std::{collections::BTreeMap, iter, ops::Div as _};

use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext, Version},
    AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_app::genesis::Content;
use penumbra_sdk_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_dex::{
    event::{EventArbExecution, EventCandlestickData},
    DirectedTradingPair,
};
use penumbra_sdk_fee::event::EventBlockFees;
use penumbra_sdk_funding::event::EventFundingStreamReward;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::event::EventDomainType;
use penumbra_sdk_shielded_pool::event::{
    EventInboundFungibleTokenTransfer, EventOutboundFungibleTokenRefund,
    EventOutboundFungibleTokenTransfer,
};
use penumbra_sdk_stake::{
    event::{EventDelegate, EventRateDataChange, EventUndelegate},
    validator::Validator,
    IdentityKey,
};

use crate::parsing::parse_content;

fn convert_ceil(amount: u64, inv_rate_bps2: u64) -> anyhow::Result<u64> {
    Ok(u64::try_from(
        (u128::from(amount) * 1_0000_0000).div_ceil(u128::from(inv_rate_bps2)),
    )?)
}

#[derive(Debug, Clone, Copy)]
struct ValidatorSupply {
    del_um: u64,
    rate_bps2: u64,
}

async fn modify_validator_supply<T>(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    ik: IdentityKey,
    f: impl FnOnce(ValidatorSupply) -> anyhow::Result<(T, ValidatorSupply)>,
) -> anyhow::Result<T> {
    let ik_text = ik.to_string();
    let supply = {
        let row: Option<(i64, i64)> = sqlx::query_as("
            SELECT del_um, rate_bps2 FROM _insights_validators WHERE validator_id = $1 ORDER BY height DESC LIMIT 1
        ").bind(&ik_text).fetch_optional(dbtx.as_mut()).await?;
        let row = row.unwrap_or((0i64, 1_0000_0000i64));
        ValidatorSupply {
            del_um: u64::try_from(row.0)?,
            rate_bps2: u64::try_from(row.1)?,
        }
    };
    let (out, new_supply) = f(supply)?;
    sqlx::query(
        r#"
        INSERT INTO _insights_validators 
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (validator_id, height) DO UPDATE SET
            del_um = excluded.del_um,
            rate_bps2 = excluded.rate_bps2
    "#,
    )
    .bind(&ik_text)
    .bind(i64::try_from(height)?)
    .bind(i64::try_from(new_supply.del_um)?)
    .bind(i64::try_from(new_supply.rate_bps2)?)
    .execute(dbtx.as_mut())
    .await?;
    Ok(out)
}

#[derive(Default, Debug, Clone, Copy)]
struct Supply {
    total: u64,
    staked: u64,
    price: Option<f64>,
}

async fn modify_supply(
    dbtx: &mut PgTransaction<'_>,
    height: u64,
    price_numeraire: Option<asset::Id>,
    f: Box<dyn FnOnce(Supply) -> anyhow::Result<Supply> + Send + 'static>,
) -> anyhow::Result<()> {
    let supply: Supply = {
        let row: Option<(i64, i64, Option<f64>)> = sqlx::query_as(
            "SELECT total, staked, price FROM insights_supply ORDER BY HEIGHT DESC LIMIT 1",
        )
        .fetch_optional(dbtx.as_mut())
        .await?;
        row.map(|(total, staked, price)| {
            anyhow::Result::<_>::Ok(Supply {
                total: total.try_into()?,
                staked: staked.try_into()?,
                price,
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
            VALUES ($1, $2, $3, $5, $4)
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
    .bind(supply.price)
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
    flow: I256,
    refund: bool,
    depositor_existed: DepositorExisted,
) -> anyhow::Result<()> {
    let asset_pool: Option<(String, String, i32)> = sqlx::query_as("SELECT total_value, current_value, unique_depositors FROM insights_shielded_pool WHERE asset_id = $1 ORDER BY height DESC LIMIT 1").bind(asset_id.to_bytes()).fetch_optional(dbtx.as_mut()).await?;
    let mut asset_pool = asset_pool
        .map(|(t, c, u)| {
            anyhow::Result::<(I256, I256, i32)>::Ok((
                I256::from_str_radix(&t, 10)?,
                I256::from_str_radix(&c, 10)?,
                u,
            ))
        })
        .transpose()?
        .unwrap_or((I256::ZERO, I256::ZERO, 0i32));
    asset_pool.0 += if refund {
        I256::ZERO
    } else {
        flow.max(I256::ZERO)
    };
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
}

impl Component {
    /// This component depends on a reference asset for the total supply price.
    pub fn new(price_numeraire: Option<asset::Id>) -> Self {
        Self { price_numeraire }
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
        modify_validator_supply(dbtx, 0, val.identity_key, move |_| {
            Ok((
                (),
                ValidatorSupply {
                    del_um: delegation_amount,
                    rate_bps2: 1_0000_0000,
                },
            ))
        })
        .await?;
    }

    modify_supply(
        dbtx,
        0,
        None,
        Box::new(move |_| {
            Ok(Supply {
                total: unstaked + staked,
                staked,
                price: None,
            })
        }),
    )
    .await?;

    Ok(())
}

impl Component {
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction<'_>,
        event: ContextualizedEvent<'_>,
    ) -> Result<(), anyhow::Error> {
        let height = event.block_height;
        if let Ok(e) = EventUndelegate::try_from_event(&event.event) {
            let amount = u64::try_from(e.amount.value())
                .context("I-000-002: undelegation amount not u64")?;
            // When the delegated um was undelegated, conversion was applied to round down,
            // so when converting back, we round up.
            modify_validator_supply(dbtx, height, e.identity_key, move |supply| {
                Ok((
                    (),
                    ValidatorSupply {
                        del_um: supply
                            .del_um
                            .checked_sub(convert_ceil(amount, supply.rate_bps2)?)
                            .ok_or(anyhow!(
                                "I-000-005: underflow of validator supply for {}",
                                e.identity_key
                            ))?,
                        ..supply
                    },
                ))
            })
            .await?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                Box::new(move |supply| {
                    // The amount staked has changed, but no inflation has happened.
                    Ok(Supply {
                        staked: supply
                            .staked
                            .checked_sub(amount)
                            .ok_or(anyhow!("I-000-001: underflow of staked supply"))?,
                        ..supply
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventDelegate::try_from_event(&event.event) {
            let amount = u64::try_from(e.amount.value())
                .context("I-000-003: undelegation amount not u64")?;
            modify_validator_supply(
                dbtx,
                height,
                e.identity_key,
                // When converting, we round down so that the user gets *less*.
                move |supply| {
                    Ok((
                        (),
                        ValidatorSupply {
                            del_um: supply
                                .del_um
                                .checked_add(convert_ceil(amount, supply.rate_bps2)?)
                                .ok_or(anyhow!(
                                    "I-000-006: overflow of validator supply for {}",
                                    e.identity_key
                                ))?,
                            ..supply
                        },
                    ))
                },
            )
            .await?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                Box::new(move |supply| {
                    Ok(Supply {
                        staked: supply
                            .staked
                            .checked_add(amount)
                            .ok_or(anyhow!("I-000-004: overflow of staked supply"))?,
                        ..supply
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventRateDataChange::try_from_event(&event.event) {
            let delta = modify_validator_supply(dbtx, height, e.identity_key, move |supply| {
                let rate_bps2 = u64::try_from(e.rate_data.validator_exchange_rate.value())?;
                let old_um =
                    (i128::from(supply.del_um) * i128::from(supply.rate_bps2)).div(1_0000_0000);
                let new_um = (i128::from(supply.del_um) * i128::from(rate_bps2)).div(1_0000_0000);
                Ok((
                    i64::try_from(new_um - old_um)?,
                    ValidatorSupply {
                        rate_bps2,
                        del_um: supply.del_um,
                    },
                ))
            })
            .await?;
            modify_supply(
                dbtx,
                height,
                self.price_numeraire,
                Box::new(move |supply| {
                    // Value has been created or destroyed!
                    Ok(Supply {
                        total: u64::try_from(i64::try_from(supply.total)? + delta)?,
                        staked: u64::try_from(i64::try_from(supply.staked)? + delta)?,
                        ..supply
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
                    Box::new(move |supply| {
                        Ok(Supply {
                            total: supply.total - amount,
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
                    Box::new(move |supply| {
                        Ok(Supply {
                            total: supply.total - profit,
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
                Box::new(move |supply| {
                    Ok(Supply {
                        total: supply.total + amount,
                        ..supply
                    })
                }),
            )
            .await?;
        } else if let Ok(e) = EventInboundFungibleTokenTransfer::try_from_event(&event.event) {
            if e.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                let existed = register_depositor(dbtx, e.value.asset_id, &e.sender).await?;
                let flow = I256::try_from(e.value.amount.value())?;
                asset_flow(dbtx, e.value.asset_id, height, flow, false, existed).await?;
            }
        } else if let Ok(e) = EventOutboundFungibleTokenTransfer::try_from_event(&event.event) {
            if e.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                let flow = I256::try_from(e.value.amount.value())?;
                // For outbound transfers, never increment unique count
                asset_flow(
                    dbtx,
                    e.value.asset_id,
                    height,
                    -flow,
                    false,
                    DepositorExisted::No,
                )
                .await?;
            }
        } else if let Ok(e) = EventOutboundFungibleTokenRefund::try_from_event(&event.event) {
            if e.value.asset_id != *STAKING_TOKEN_ASSET_ID {
                let flow = I256::try_from(e.value.amount.value())?;
                // For outbound transfers, never increment unique count.
                asset_flow(
                    dbtx,
                    e.value.asset_id,
                    height,
                    flow,
                    true,
                    DepositorExisted::No,
                )
                .await?;
            }
        } else if let Ok(e) = EventCandlestickData::try_from_event(&event.event) {
            if let Some(pn) = self.price_numeraire {
                if e.pair == DirectedTradingPair::new(*STAKING_TOKEN_ASSET_ID, pn) {
                    let price = e.stick.close;
                    modify_supply(
                        dbtx,
                        height,
                        self.price_numeraire,
                        Box::new(move |supply| {
                            Ok(Supply {
                                price: Some(price),
                                ..supply
                            })
                        }),
                    )
                    .await?;
                }
            }
        }
        tracing::debug!(?event, "unrecognized event");
        Ok(())
    }
}

#[async_trait]
impl AppView for Component {
    fn version(&self) -> Version {
        let hash: [u8; 32] = blake2b_simd::Params::default()
            .personal(b"option_hash")
            .hash_length(32)
            .to_state()
            .update(
                self.price_numeraire
                    .map(|x| x.to_bytes())
                    .unwrap_or_default()
                    .as_slice(),
            )
            .finalize()
            .as_bytes()
            .try_into()
            .expect("Impossible 000-002: expected 32 byte hash");
        Version::default().with_option_hash(hash)
    }

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

    fn name(&self) -> String {
        "insights".to_string()
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        for event in batch.events() {
            self.index_event(dbtx, event).await?;
        }
        Ok(())
    }

    async fn reset(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("reset.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }
}
