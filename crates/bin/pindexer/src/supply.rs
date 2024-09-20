use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_app::genesis::AppState;
use penumbra_asset::asset;
use penumbra_keys::Address;
use penumbra_num::Amount;
use penumbra_proto::{event::ProtoEvent, penumbra::core::component::funding::v1 as pb};
use penumbra_stake::{rate::RateData, validator::Validator, IdentityKey};
use sqlx::{types::chrono::DateTime, PgPool, Postgres, Transaction};
use std::str::FromStr;

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
    FundingStreamReward {
        height: u64,
        recipient: Address,
        epoch_index: u64,
        reward_amount: Amount,
    },
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
        "penumbra.core.component.stake.v1.EventRateDataChange",
        "penumbra.core.component.funding.v1.EventFundingStreamReward",
    ];

    async fn index<'d>(&self, dbtx: &mut Transaction<'d, Postgres>) -> anyhow::Result<()> {
        match self {
            Event::Delegate {
                height,
                identity_key,
                amount,
            } => {
                let previous_total_um: i64 = sqlx::query_scalar(
                    r#"SELECT total_um FROM supply_total_unstaked ORDER BY height DESC LIMIT 1"#,
                )
                .fetch_optional(dbtx.as_mut())
                .await?
                .unwrap_or(0);

                let new_total_um = previous_total_um - amount.value() as i64;

                sqlx::query(
                    r#"INSERT INTO supply_total_unstaked (height, total_um) VALUES ($1, $2)"#,
                )
                .bind(*height as i64)
                .bind(new_total_um)
                .execute(dbtx.as_mut())
                .await?;

                // Update supply_total_staked
                let prev_um_eq_delegations: i64 = sqlx::query_scalar(
                    r"SELECT um_equivalent_delegations FROM supply_total_staked WHERE validator_id = $1 ORDER BY height DESC LIMIT 1"
                )
                .bind(identity_key.to_string())
                .fetch_optional(dbtx.as_mut())
                .await?
                .unwrap_or(0);
                let prev_delegations: i64 = sqlx::query_scalar(
                    "SELECT delegations FROM supply_total_staked WHERE validator_id = $1 ORDER BY height DESC LIMIT 1"
                )
                .bind(identity_key.to_string())
                .fetch_optional(dbtx.as_mut())
                .await?
                .unwrap_or(0);

                let new_um_eq_delegations = prev_um_eq_delegations + amount.value() as i64;

                let new_delegations = if prev_um_eq_delegations == 0 {
                    amount.value() as i64
                } else {
                    let delta_delegations = (amount.value() as f64 / prev_um_eq_delegations as f64)
                        * prev_delegations as f64;
                    prev_delegations + delta_delegations as i64
                };

                sqlx::query(
                    r#"INSERT INTO supply_total_staked (height, validator_id, um_equivalent_delegations, delegations) VALUES ($1, $2, $3, $4)"#,
                )
                .bind(*height as i64)
                .bind(identity_key.to_string())
                .bind(new_um_eq_delegations)
                .bind(new_delegations)
                .execute(dbtx.as_mut())
                .await?;

                Ok(())
            }
            Event::Undelegate {
                height,
                identity_key,
                unbonded_amount,
            } => {
                let previous_total_um: i64 = sqlx::query_scalar(
                    r#"SELECT total_um FROM supply_total_unstaked ORDER BY height DESC LIMIT 1"#,
                )
                .fetch_optional(dbtx.as_mut())
                .await?
                .unwrap_or(0);

                let new_total_um = previous_total_um + unbonded_amount.value() as i64;

                sqlx::query(
                    r#"INSERT INTO supply_total_unstaked (height, total_um) VALUES ($1, $2)"#,
                )
                .bind(*height as i64)
                .bind(new_total_um)
                .execute(dbtx.as_mut())
                .await?;

                let prev_um_eq_delegations: i64 = sqlx::query_scalar(
                    r"SELECT um_equivalent_delegations FROM supply_total_staked WHERE validator_id = $1 ORDER BY height DESC LIMIT 1"
                )
                .bind(identity_key.to_string())
                .fetch_optional(dbtx.as_mut())
                .await?
                .unwrap_or(0);

                let prev_delegations: i64 = sqlx::query_scalar(
                    "SELECT delegations FROM supply_total_staked WHERE validator_id = $1 ORDER BY height DESC LIMIT 1"
                )
                .bind(identity_key.to_string())
                .fetch_optional(dbtx.as_mut())
                .await?
                .unwrap_or(0);

                let new_um_eq_delegations = prev_um_eq_delegations - unbonded_amount.value() as i64;

                if prev_um_eq_delegations == 0 {
                    return Err(anyhow::anyhow!(
                        "Previous um_equivalent_delegations is zero"
                    ));
                }

                let delta_delegations = (unbonded_amount.value() as f64
                    / prev_um_eq_delegations as f64)
                    * prev_delegations as f64;
                let new_delegations = prev_delegations - delta_delegations as i64;

                sqlx::query(
                    r#"INSERT INTO supply_total_staked (height, validator_id, um_equivalent_delegations, delegations) VALUES ($1, $2, $3, $4)"#,
                )
                .bind(*height as i64)
                .bind(identity_key.to_string())
                .bind(new_um_eq_delegations)
                .bind(new_delegations)
                .execute(dbtx.as_mut())
                .await?;

                Ok(())
            }
            Event::FundingStreamReward {
                height,
                recipient: _,
                epoch_index: _,
                reward_amount,
            } => {
                let prev_unstaked: i64 = sqlx::query_scalar(
                    "SELECT total_um FROM supply_total_unstaked ORDER BY height DESC LIMIT 1",
                )
                .fetch_optional(dbtx.as_mut())
                .await?
                .ok_or(anyhow!("couldnt look up the previous supply"))?;

                let new_unstaked = prev_unstaked + reward_amount.value() as i64;

                sqlx::query("INSERT INTO supply_total_unstaked (height, total_um) VALUES ($1, $2)")
                    .bind(*height as i64)
                    .bind(new_unstaked)
                    .execute(dbtx.as_mut())
                    .await?;

                Ok(())
            }
            Event::RateDataChange {
                height,
                identity_key,
                rate_data,
            } => {
                

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
            x if x == Event::NAMES[0] => {}
            // delegation
            x if x == Event::NAMES[1] => {}
            // funding stream reward
            x if x == Event::NAMES[2] => {
                let pe = pb::EventFundingStreamReward::from_event(event.as_ref())?;
                let recipient = Address::from_str(&pe.recipient)?;
                let epoch_index = pe.epoch_index;
                let reward_amount = Amount::try_from(
                    pe.reward_amount
                        .ok_or(anyhow!("event missing in funding stream reward"))?,
                )?;
                Ok(Self::FundingStreamReward {
                    height: event.block_height,
                    recipient,
                    epoch_index,
                    reward_amount,
                })
            }
            // validator rate change
            x: if x == Event::NAMES[3] => {},
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

    sqlx::query("INSERT INTO supply_total_unstaked (height, total_um) VALUES ($1, $2)")
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

        sqlx::query("INSERT INTO supply_total_staked (height, validator_id, um_equivalent_delegations, delegations) VALUES ($1, $2, $3, $4)")
            .bind(0i64)
            .bind(val.identity_key.to_string())
            .bind(delegation_amount.value() as i64)
            .bind(delegation_amount.value() as i64)
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
    total_um BIGINT NOT NULL,
);

CREATE TABLE IF NOT EXISTS supply_total_staked (
    height BIGINT PRIMARY KEY,
    validator_id TEXT NOT NULL,
    um_equivalent_delegations BIGINT NOT NULL,
    delegations BIGINT NOT NULL,
)
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
