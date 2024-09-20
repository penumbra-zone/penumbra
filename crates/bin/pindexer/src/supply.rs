use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_app::genesis::AppState;
use penumbra_asset::asset;
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{
    event::ProtoEvent, penumbra::core::component::funding::v1 as pb_funding,
    penumbra::core::component::stake::v1 as pb_stake,
};
use penumbra_stake::{rate::RateData, validator::Validator, IdentityKey};
use sqlx::{PgPool, Postgres, Transaction};

const BPS_SQUARED: u64 = 1_0000_0000u64;

#[derive(Clone, Copy)]
struct DelegatedSupply {
    um: u64,
    del_um: u64,
    rate_bps2: u64,
}

impl Default for DelegatedSupply {
    fn default() -> Self {
        Self {
            um: 0,
            del_um: 0,
            rate_bps2: BPS_SQUARED,
        }
    }
}

impl DelegatedSupply {
    fn modify<const NEGATE: bool>(self, delta: u64) -> anyhow::Result<Self> {
        let rate = U128x128::ratio(self.rate_bps2, BPS_SQUARED)?;
        let um_delta = delta;
        let del_um_delta = if rate == U128x128::from(0u128) {
            0u64
        } else {
            let del_um_delta = (U128x128::from(delta) / rate)?;
            let rounded = if NEGATE {
                // So that we don't remove too few del_um
                del_um_delta.round_up()?
            } else {
                // So that we don't add too many del_um
                del_um_delta.round_down()
            };
            rounded.try_into()?
        };
        let out = if NEGATE {
            Self {
                um: self
                    .um
                    .checked_add(um_delta)
                    .ok_or(anyhow!("supply modification failed"))?,
                del_um: self
                    .del_um
                    .checked_add(del_um_delta)
                    .ok_or(anyhow!("supply modification failed"))?,
                rate_bps2: self.rate_bps2,
            }
        } else {
            Self {
                um: self
                    .um
                    .checked_add(um_delta)
                    .ok_or(anyhow!("supply modification failed"))?,
                del_um: self
                    .del_um
                    .checked_add(del_um_delta)
                    .ok_or(anyhow!("supply modification failed"))?,
                rate_bps2: self.rate_bps2,
            }
        };
        Ok(out)
    }

    fn rate_change(self, rate_data: &RateData) -> Result<Self> {
        let um = rate_data
            .unbonded_amount(self.del_um.into())
            .value()
            .try_into()?;

        Ok(Self {
            um,
            del_um: self.del_um,
            rate_bps2: rate_data.validator_exchange_rate.value().try_into()?,
        })
    }
}

async fn add_validator<'d>(
    dbtx: &mut Transaction<'d, Postgres>,
    identity_key: &IdentityKey,
) -> anyhow::Result<i32> {
    let ik_string = identity_key.to_string();
    let id: Option<i32> =
        sqlx::query_scalar(r#"SELECT id FROM supply_validators WHERE identity_key = $1"#)
            .bind(&ik_string)
            .fetch_optional(dbtx.as_mut())
            .await?;
    if let Some(id) = id {
        return Ok(id);
    }
    let id =
        sqlx::query_scalar(r#"INSERT INTO supply_validators VALUES (DEFAULT, $1) RETURNING id"#)
            .bind(&ik_string)
            .fetch_one(dbtx.as_mut())
            .await?;
    Ok(id)
}

async fn delegated_supply_current<'d>(
    dbtx: &mut Transaction<'d, Postgres>,
    val_id: i32,
) -> Result<Option<DelegatedSupply>> {
    let row: Option<(i64, i64, i64)> = sqlx::query_as("SELECT um, del_um, rate_bps2 FROM supply_total_staked WHERE validator_id = $1 ORDER BY height DESC LIMIT 1")
        .bind(val_id).fetch_optional(dbtx.as_mut()).await?;
    row.map(|(um, del_um, rate_bps2)| {
        let um = um.try_into()?;
        let del_um = del_um.try_into()?;
        let rate_bps2 = rate_bps2.try_into()?;
        Ok(DelegatedSupply {
            um,
            del_um,
            rate_bps2,
        })
    })
    .transpose()
}

async fn supply_current<'d>(dbtx: &mut Transaction<'d, Postgres>) -> Result<u64> {
    let row: Option<i64> =
        sqlx::query_scalar("SELECT um FROM supply_total_unstaked ORDER BY height DESC LIMIT 1")
            .fetch_optional(dbtx.as_mut())
            .await?;
    Ok(row.unwrap_or_default().try_into()?)
}

/// Supply-relevant events.
/// The supply of the native staking token can change:
/// - When notes are minted (e.g., during initial genesis, or as a result of
/// IBC, though in the case of IBC the circuit breaker should never allow more
/// inbound UM to be minted than outbound um were originally sent.)
/// - As a result of claiming delegation tokens that have increased in
/// underlying UM value due to accumulating the staking rate.
/// - As a result of burning UM which can happen due to arbs, fees, and slashing.
#[derive(Clone, Debug)]
enum Event {
    /// A parsed version of [pb::EventUndelegate]
    Undelegate {
        height: u64,
        identity_key: IdentityKey,
        unbonded_amount: Amount,
    },
    /// A parsed version of [pb::EventDelegate]
    Delegate {
        height: u64,
        identity_key: IdentityKey,
        amount: Amount,
    },
    /// A parsed version of [pb::EventFundingStreamReward]
    FundingStreamReward { height: u64, reward_amount: Amount },
    /// A parsed version of EventRateDataChange
    RateDataChange {
        height: u64,
        identity_key: IdentityKey,
        rate_data: RateData,
    },
}

impl Event {
    const NAMES: [&'static str; 4] = [
        "penumbra.core.component.stake.v1.EventUndelegate",
        "penumbra.core.component.stake.v1.EventDelegate",
        "penumbra.core.component.funding.v1.EventFundingStreamReward",
        "penumbra.core.component.stake.v1.EventRateDataChange",
    ];

    async fn index<'d>(&self, dbtx: &mut Transaction<'d, Postgres>) -> anyhow::Result<()> {
        match self {
            Event::Delegate {
                height,
                identity_key,
                amount,
            } => {
                let amount = u64::try_from(amount.value())?;

                let val_id = add_validator(dbtx, identity_key).await?;
                let current_supply = delegated_supply_current(dbtx, val_id).await?;
                let new_supply = current_supply.unwrap_or_default().modify::<false>(amount)?;
                sqlx::query(
                    r#"
                    INSERT INTO
                        supply_total_staked
                    VALUES ($1, $2, $3, $4, $5) 
                    ON CONFLICT (validator_id, height)
                    DO UPDATE SET
                        um = excluded.um,
                        del_um = excluded.del_um,
                        rate_bps2 = excluded.rate_bps2
                "#,
                )
                .bind(val_id)
                .bind(i64::try_from(*height)?)
                .bind(i64::try_from(new_supply.um)?)
                .bind(i64::try_from(new_supply.del_um)?)
                .bind(i64::try_from(new_supply.rate_bps2)?)
                .execute(dbtx.as_mut())
                .await?;
                let current_um = supply_current(dbtx).await?;
                let new_um = current_um
                    .checked_sub(amount)
                    .ok_or(anyhow!("um supply underflowed"))?;
                sqlx::query(
                    r#"
                    INSERT INTO
                        supply_total_unstaked
                    VALUES ($1, $2) 
                    ON CONFLICT (height)
                    DO UPDATE SET
                        um = excluded.um
                "#,
                )
                .bind(i64::try_from(*height)?)
                .bind(i64::try_from(new_um)?)
                .execute(dbtx.as_mut())
                .await?;

                Ok(())
            }
            Event::Undelegate {
                height,
                identity_key,
                unbonded_amount,
            } => {
                let amount = u64::try_from(unbonded_amount.value())?;

                let val_id = add_validator(dbtx, identity_key).await?;
                let current_supply = delegated_supply_current(dbtx, val_id).await?;
                let new_supply = current_supply.unwrap_or_default().modify::<true>(amount)?;
                sqlx::query(
                    r#"
                    INSERT INTO
                        supply_total_staked
                    VALUES ($1, $2, $3, $4, $5) 
                    ON CONFLICT (validator_id, height)
                    DO UPDATE SET
                        um = excluded.um,
                        del_um = excluded.del_um,
                        rate_bps2 = excluded.rate_bps2
                "#,
                )
                .bind(val_id)
                .bind(i64::try_from(*height)?)
                .bind(i64::try_from(new_supply.um)?)
                .bind(i64::try_from(new_supply.del_um)?)
                .bind(i64::try_from(new_supply.rate_bps2)?)
                .execute(dbtx.as_mut())
                .await?;
                let current_um = supply_current(dbtx).await?;
                let new_um = current_um
                    .checked_add(amount)
                    .ok_or(anyhow!("um supply overflowed"))?;
                sqlx::query(
                    r#"
                    INSERT INTO
                        supply_total_unstaked
                    VALUES ($1, $2) 
                    ON CONFLICT (height)
                    DO UPDATE SET
                        um = excluded.um
                "#,
                )
                .bind(i64::try_from(*height)?)
                .bind(i64::try_from(new_um)?)
                .execute(dbtx.as_mut())
                .await?;

                Ok(())
            }
            Event::FundingStreamReward {
                height,
                reward_amount,
            } => {
                let amount = u64::try_from(reward_amount.value())?;
                let current_um = supply_current(dbtx).await?;
                let new_um = current_um
                    .checked_add(amount)
                    .ok_or(anyhow!("um supply overflowed"))?;
                sqlx::query(
                    r#"
                    INSERT INTO
                        supply_total_unstaked
                    VALUES ($1, $2) 
                    ON CONFLICT (height)
                    DO UPDATE SET
                        um = excluded.um
                "#,
                )
                .bind(i64::try_from(*height)?)
                .bind(i64::try_from(new_um)?)
                .execute(dbtx.as_mut())
                .await?;

                Ok(())
            }
            Event::RateDataChange {
                height,
                identity_key,
                rate_data,
            } => {
                let val_id = add_validator(dbtx, identity_key).await?;
                let current_supply = delegated_supply_current(dbtx, val_id).await?;
                let new_supply = current_supply.unwrap_or_default().rate_change(rate_data)?;
                sqlx::query(
                    r#"
                    INSERT INTO
                        supply_total_staked
                    VALUES ($1, $2, $3, $4, $5) 
                    ON CONFLICT (validator_id, height)
                    DO UPDATE SET
                        um = excluded.um,
                        del_um = excluded.del_um,
                        rate_bps2 = excluded.rate_bps2
                "#,
                )
                .bind(val_id)
                .bind(i64::try_from(*height)?)
                .bind(i64::try_from(new_supply.um)?)
                .bind(i64::try_from(new_supply.del_um)?)
                .bind(i64::try_from(new_supply.rate_bps2)?)
                .execute(dbtx.as_mut())
                .await?;

                Ok(())
            }
        }
    }
}

impl<'a> TryFrom<&'a ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &'a ContextualizedEvent) -> Result<Self, Self::Error> {
        match event.event.kind.as_str() {
            // undelegation
            x if x == Event::NAMES[0] => {
                let pe = pb_stake::EventUndelegate::from_event(event.as_ref())?;
                let identity_key = pe
                    .identity_key
                    .ok_or(anyhow!("EventUndelegate should contain identity key"))?
                    .try_into()?;
                let unbonded_amount = pe
                    .amount
                    .ok_or(anyhow!("EventUndelegate should contain amount"))?
                    .try_into()?;
                Ok(Self::Undelegate {
                    height: event.block_height,
                    identity_key,
                    unbonded_amount,
                })
            }
            // delegation
            x if x == Event::NAMES[1] => {
                let pe = pb_stake::EventDelegate::from_event(event.as_ref())?;
                let identity_key = pe
                    .identity_key
                    .ok_or(anyhow!("EventDelegate should contain identity key"))?
                    .try_into()?;
                let amount = pe
                    .amount
                    .ok_or(anyhow!("EventDelegate should contain amount"))?
                    .try_into()?;
                Ok(Self::Delegate {
                    height: event.block_height,
                    identity_key,
                    amount,
                })
            }
            // funding stream reward
            x if x == Event::NAMES[2] => {
                let pe = pb_funding::EventFundingStreamReward::from_event(event.as_ref())?;
                let reward_amount = Amount::try_from(
                    pe.reward_amount
                        .ok_or(anyhow!("event missing in funding stream reward"))?,
                )?;
                Ok(Self::FundingStreamReward {
                    height: event.block_height,
                    reward_amount,
                })
            }
            // validator rate change
            x if x == Event::NAMES[3] => {
                let pe = pb_stake::EventRateDataChange::from_event(event.as_ref())?;
                let identity_key = pe
                    .identity_key
                    .ok_or(anyhow!("EventRateDataChange should contain identity key"))?
                    .try_into()?;
                let rate_data = pe
                    .rate_data
                    .ok_or(anyhow!("EventRateDataChange should contain rate data"))?
                    .try_into()?;
                Ok(Self::RateDataChange {
                    height: event.block_height,
                    identity_key,
                    rate_data,
                })
            }
            x => Err(anyhow!(format!("unrecognized event kind: {x}"))),
        }
    }
}

/// Add the initial native token supply.
async fn add_genesis_native_token_allocation_supply<'a>(
    dbtx: &mut PgTransaction<'a>,
    app_state: &AppState,
) -> Result<()> {
    let content = app_state
        .content()
        .ok_or_else(|| anyhow::anyhow!("cannot initialized indexer from checkpoint genesis"))?;

    let mut unstaked_native_token_sum: Amount = Amount::zero();
    for allo in &content.shielded_pool_content.allocations {
        if allo.denom().base_denom().denom == "upenumbra" {
            let value = allo.value();
            unstaked_native_token_sum = unstaked_native_token_sum
                .checked_add(&value.amount)
                .unwrap();
        }
    }

    sqlx::query("INSERT INTO supply_total_unstaked (height, um) VALUES ($1, $2)")
        .bind(0i64)
        .bind(unstaked_native_token_sum.value() as i64)
        .execute(dbtx.as_mut())
        .await?;

    let mut allos = BTreeMap::<asset::Id, Amount>::new();
    for allo in &content.shielded_pool_content.allocations {
        let value = allo.value();
        let sum = allos.entry(value.asset_id).or_default();
        *sum = sum
            .checked_add(&value.amount)
            .ok_or_else(|| anyhow::anyhow!("overflow adding genesis allos (should not happen)"))?;
    }

    // at genesis, assume a 1:1 ratio between delegation amount and native token amount.
    for val in &content.stake_content.validators {
        let val = Validator::try_from(val.clone())?;
        let delegation_amount = allos.get(&val.token().id()).cloned().unwrap_or_default();

        let val_id = add_validator(dbtx, &val.identity_key).await?;

        sqlx::query("INSERT INTO supply_total_staked (height, validator_id, um, del_um, rate_bps2) VALUES ($1, $2, $3, $4, $5)")
            .bind(0i64)
            .bind(val_id)
            .bind(delegation_amount.value() as i64)
            .bind(delegation_amount.value() as i64)
            .bind(BPS_SQUARED as i64)
            .execute(dbtx.as_mut())
            .await?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct Supply {
    event_strings: HashSet<&'static str>,
}

impl Supply {
    pub fn new() -> Self {
        let event_strings = Event::NAMES.into_iter().collect();
        Self { event_strings }
    }
}

#[async_trait]
impl AppView for Supply {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            // table name is module path + struct name
            "
CREATE TABLE IF NOT EXISTS supply_total_unstaked (
    height BIGINT PRIMARY KEY,
    um BIGINT NOT NULL
);
",
        )
        .execute(dbtx.as_mut())
        .await?;
        sqlx::query(
            // table name is module path + struct name
            "
CREATE TABLE IF NOT EXISTS supply_validators (
    id SERIAL PRIMARY KEY,
    identity_key TEXT NOT NULL
);
",
        )
        .execute(dbtx.as_mut())
        .await?;
        sqlx::query(
            "
CREATE TABLE IF NOT EXISTS supply_total_staked (
    validator_id INT REFERENCES supply_validators(id),
    height BIGINT NOT NULL,
    um BIGINT NOT NULL,
    del_um BIGINT NOT NULL,
    rate_bps2 BIGINT NOT NULL,
    PRIMARY KEY (validator_id, height)
);
",
        )
        .execute(dbtx.as_mut())
        .await?;

        // decode the initial supply from the genesis
        // initial app state is not recomputed from events, because events are not emitted in init_chain.
        // instead, the indexer directly parses the genesis.
        let app_state: penumbra_app::genesis::AppState =
            serde_json::from_value(app_state.clone()).context("error decoding app_state json")?;
        add_genesis_native_token_allocation_supply(dbtx, &app_state).await?;

        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        self.event_strings.contains(type_str)
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        Event::try_from(event)?.index(dbtx).await
    }
}
