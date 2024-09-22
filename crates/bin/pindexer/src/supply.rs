use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_app::genesis::{AppState, Content};
use penumbra_asset::{asset, STAKING_TOKEN_ASSET_ID};
use penumbra_num::Amount;
use penumbra_proto::{
    event::ProtoEvent, penumbra::core::component::funding::v1 as pb_funding,
    penumbra::core::component::stake::v1 as pb_stake,
};
use penumbra_stake::{rate::RateData, validator::Validator, IdentityKey};
use sqlx::{PgPool, Postgres, Transaction};
use std::iter;

mod unstaked_supply {
    //! This module handles updates around the unstaked supply.
    use anyhow::Result;
    use cometindex::PgTransaction;

    /// Initialize the database tables for this module.
    pub async fn init_db(dbtx: &mut PgTransaction<'_>) -> Result<()> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS supply_total_unstaked (
            height BIGINT PRIMARY KEY,
            um BIGINT NOT NULL
        );
        "#,
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    /// Get the supply for at a given height.
    async fn get_supply(dbtx: &mut PgTransaction<'_>, height: u64) -> Result<Option<u64>> {
        let row: Option<i64> = sqlx::query_scalar(
            "SELECT um FROM supply_total_unstaked WHERE height <= $1 ORDER BY height DESC LIMIT 1",
        )
        .bind(i64::try_from(height)?)
        .fetch_optional(dbtx.as_mut())
        .await?;
        row.map(|x| u64::try_from(x))
            .transpose()
            .map_err(Into::into)
    }

    /// Set the supply at a given height.
    async fn set_supply(dbtx: &mut PgTransaction<'_>, height: u64, supply: u64) -> Result<()> {
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
        .bind(i64::try_from(height)?)
        .bind(i64::try_from(supply)?)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    /// Modify the supply at a given height.
    ///
    /// This will take the supply at the given height, and replace it with the
    /// new result produced by the function.
    pub async fn modify(
        dbtx: &mut PgTransaction<'_>,
        height: u64,
        f: impl FnOnce(Option<u64>) -> Result<u64>,
    ) -> Result<()> {
        let supply = get_supply(dbtx, height).await?;
        let new_supply = f(supply)?;
        set_supply(dbtx, height, new_supply).await
    }
}

mod delegated_supply {
    //! This module handles updates around the delegated supply to a validator.
    use anyhow::{anyhow, Result};
    use cometindex::PgTransaction;
    use penumbra_num::fixpoint::U128x128;
    use penumbra_stake::{rate::RateData, IdentityKey};

    const BPS_SQUARED: u64 = 1_0000_0000u64;

    /// Represents the supply around a given validator.
    ///
    /// The supply needs to track the amount of delegated tokens to that validator,
    /// as well as the conversion rate from those tokens to the native token.
    #[derive(Clone, Copy)]
    pub struct Supply {
        um: u64,
        del_um: u64,
        rate_bps2: u64,
    }

    impl Default for Supply {
        fn default() -> Self {
            Self {
                um: 0,
                del_um: 0,
                rate_bps2: BPS_SQUARED,
            }
        }
    }

    impl Supply {
        /// Change the amount of um in this supply, by adding or removing um.
        pub fn add_um(self, delta: i64) -> Result<Self> {
            let rate = U128x128::ratio(self.rate_bps2, BPS_SQUARED)?;
            let negate = delta.is_negative();
            let delta = delta.unsigned_abs();
            let um_delta = delta;
            let del_um_delta = if rate == U128x128::from(0u128) {
                0u64
            } else {
                let del_um_delta = (U128x128::from(delta) / rate)?;
                let rounded = if negate {
                    // So that we don't remove too few del_um
                    del_um_delta.round_up()?
                } else {
                    // So that we don't add too many del_um
                    del_um_delta.round_down()
                };
                rounded.try_into()?
            };
            let out = if negate {
                Self {
                    um: self
                        .um
                        .checked_sub(um_delta)
                        .ok_or(anyhow!("supply modification failed"))?,
                    del_um: self
                        .del_um
                        .checked_sub(del_um_delta)
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

        /// Change the conversion rate between delegated_um and um in this supply.
        pub fn change_rate(self, rate: &RateData) -> Result<Self> {
            let um = rate
                .unbonded_amount(self.del_um.into())
                .value()
                .try_into()?;

            Ok(Self {
                um,
                del_um: self.del_um,
                rate_bps2: rate.validator_exchange_rate.value().try_into()?,
            })
        }
    }

    /// Initialize the database tables for this module.
    pub async fn init_db<'d>(dbtx: &mut PgTransaction<'d>) -> Result<()> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS supply_validators (
            id SERIAL PRIMARY KEY,
            identity_key TEXT NOT NULL
        );
        "#,
        )
        .execute(dbtx.as_mut())
        .await?;
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS supply_total_staked (
            validator_id INT REFERENCES supply_validators(id),
            height BIGINT NOT NULL,
            um BIGINT NOT NULL,
            del_um BIGINT NOT NULL,
            rate_bps2 BIGINT NOT NULL,
            PRIMARY KEY (validator_id, height)
        );
        "#,
        )
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    /// An opaque internal identifier for a given validator.
    #[derive(Clone, Copy, PartialEq)]
    pub struct ValidatorID(i32);

    /// Define a validator, returning its internal ID.
    ///
    /// This will have no effect if the validator has already been defined.
    pub async fn define_validator(
        dbtx: &mut PgTransaction<'_>,
        identity_key: &IdentityKey,
    ) -> Result<ValidatorID> {
        let ik_string = identity_key.to_string();

        let id: Option<i32> =
            sqlx::query_scalar(r#"SELECT id FROM supply_validators WHERE identity_key = $1"#)
                .bind(&ik_string)
                .fetch_optional(dbtx.as_mut())
                .await?;

        if let Some(id) = id {
            return Ok(ValidatorID(id));
        }
        let id = sqlx::query_scalar(
            r#"INSERT INTO supply_validators VALUES (DEFAULT, $1) RETURNING id"#,
        )
        .bind(&ik_string)
        .fetch_one(dbtx.as_mut())
        .await?;
        Ok(ValidatorID(id))
    }

    /// Get the supply for a given validator at a given height.
    async fn get_supply(
        dbtx: &mut PgTransaction<'_>,
        validator: ValidatorID,
        height: u64,
    ) -> Result<Option<Supply>> {
        let row: Option<(i64, i64, i64)> = sqlx::query_as(
            r#"
            SELECT
                um, del_um, rate_bps2
            FROM
                supply_total_staked
            WHERE
                validator_id = $1 AND height <= $2
            ORDER BY height DESC
            LIMIT 1
        "#,
        )
        .bind(validator.0)
        .bind(i64::try_from(height)?)
        .fetch_optional(dbtx.as_mut())
        .await?;
        row.map(|(um, del_um, rate_bps2)| {
            let um = um.try_into()?;
            let del_um = del_um.try_into()?;
            let rate_bps2 = rate_bps2.try_into()?;
            Ok(Supply {
                um,
                del_um,
                rate_bps2,
            })
        })
        .transpose()
    }

    /// Set the supply for a given validator at a given height.
    async fn set_supply(
        dbtx: &mut PgTransaction<'_>,
        validator: ValidatorID,
        height: u64,
        supply: Supply,
    ) -> Result<()> {
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
        .bind(validator.0)
        .bind(i64::try_from(height)?)
        .bind(i64::try_from(supply.um)?)
        .bind(i64::try_from(supply.del_um)?)
        .bind(i64::try_from(supply.rate_bps2)?)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    /// Modify the supply for a given validator, at a given height.
    pub async fn modify(
        dbtx: &mut PgTransaction<'_>,
        validator: ValidatorID,
        height: u64,
        f: impl FnOnce(Option<Supply>) -> Result<Supply>,
    ) -> Result<()> {
        let supply = get_supply(dbtx, validator, height).await?;
        let new_supply = f(supply)?;
        set_supply(dbtx, validator, height, new_supply).await
    }
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
                let amount = i64::try_from(amount.value())?;

                unstaked_supply::modify(dbtx, *height, |current| {
                    Ok(current.unwrap_or_default() - amount as u64)
                })
                .await?;

                let validator = delegated_supply::define_validator(dbtx, identity_key).await?;
                delegated_supply::modify(dbtx, validator, *height, |current| {
                    current.unwrap_or_default().add_um(amount)
                })
                .await
            }
            Event::Undelegate {
                height,
                identity_key,
                unbonded_amount,
            } => {
                let amount = i64::try_from(unbonded_amount.value())?;

                unstaked_supply::modify(dbtx, *height, |current| {
                    Ok(current.unwrap_or_default() + amount as u64)
                })
                .await?;

                let validator = delegated_supply::define_validator(dbtx, identity_key).await?;
                delegated_supply::modify(dbtx, validator, *height, |current| {
                    current.unwrap_or_default().add_um(-amount)
                })
                .await
            }
            Event::FundingStreamReward {
                height,
                reward_amount,
            } => {
                let amount = u64::try_from(reward_amount.value())?;

                unstaked_supply::modify(dbtx, *height, |current| {
                    Ok(current.unwrap_or_default() + amount)
                })
                .await
            }
            Event::RateDataChange {
                height,
                identity_key,
                rate_data,
            } => {
                let validator = delegated_supply::define_validator(dbtx, identity_key).await?;
                delegated_supply::modify(dbtx, validator, *height, |current| {
                    current.unwrap_or_default().change_rate(rate_data)
                })
                .await
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

    let content = app_state
        .content()
        .ok_or_else(|| anyhow::anyhow!("cannot initialized indexer from checkpoint genesis"))?;
    let mints = content_mints(content);

    let unstaked_mint = u64::try_from(
        mints
            .get(&*STAKING_TOKEN_ASSET_ID)
            .copied()
            .unwrap_or_default()
            .value(),
    )?;
    unstaked_supply::modify(dbtx, 0, |_| Ok(unstaked_mint)).await?;

    // at genesis, assume a 1:1 ratio between delegation amount and native token amount.
    for val in &content.stake_content.validators {
        let val = Validator::try_from(val.clone())?;
        let delegation_amount: i64 = mints
            .get(&val.token().id())
            .cloned()
            .unwrap_or_default()
            .value()
            .try_into()?;

        let val_id = delegated_supply::define_validator(dbtx, &val.identity_key).await?;
        delegated_supply::modify(dbtx, val_id, 0, |_| {
            delegated_supply::Supply::default().add_um(delegation_amount)
        })
        .await?;
    }

    Ok(())
}

#[derive(Debug)]
pub struct Component {
    event_strings: HashSet<&'static str>,
}

impl Component {
    pub fn new() -> Self {
        let event_strings = Event::NAMES.into_iter().collect();
        Self { event_strings }
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        unstaked_supply::init_db(dbtx).await?;
        delegated_supply::init_db(dbtx).await?;

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
