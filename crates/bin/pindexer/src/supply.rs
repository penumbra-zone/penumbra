use std::collections::{BTreeMap, HashSet};

use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_app::genesis::AppState;
use penumbra_asset::asset;
use penumbra_num::Amount;
use penumbra_proto::{core::component::sct::v1 as pb, event::ProtoEvent};
use penumbra_stake::{validator::Validator, IdentityKey};
use sqlx::{types::chrono::DateTime, PgPool, Postgres, Transaction};

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
        identity_key: IdentityKey,
        unbonded_amount: Amount,
    },
    /// A parsed version of [pb::EventDelegate]
    Delegate {
        identity_key: IdentityKey,
        amount: Amount,
    }, // TKTK....
}

impl Event {
    const NAMES: [&'static str; 2] = [
        "penumbra.core.component.stake.v1.EventUndelegate",
        "penumbra.core.component.stake.v1.EventDelegate",
    ];

    async fn index<'d>(&self, dbtx: &mut Transaction<'d, Postgres>) -> anyhow::Result<()> {
        match self {
            Event::Delegate {
                identity_key: _,
                amount: _,
            } => Ok(()),
            Event::Undelegate {
                identity_key: _,
                unbonded_amount: _,
            } => Ok(()),
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

    let mut native_token_sum: Amount = Amount::zero();
    for allo in &content.shielded_pool_content.allocations {
        if allo.denom().base_denom().denom == "upenumbra" {
            let value = allo.value();
            native_token_sum = native_token_sum.checked_add(&value.amount).unwrap();
        }
    }

    // Given a genesis validator, we need to figure out its delegations at
    // genesis by getting its delegation token then summing up all the allocations.
    // Build up a table of the total allocations first.
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
        // FIXME: this shouldn't be a proto type but now that has been propagated
        // all through the rest of the code for no reason
        let val = Validator::try_from(val.clone())?;
        let delegation_amount = allos.get(&val.token().id()).cloned().unwrap_or_default();
        native_token_sum = native_token_sum + delegation_amount;
    }

    sqlx::query("INSERT INTO supply_initial_genesis (value) VALUES ($1)")
        .bind(native_token_sum.value() as i64)
        .execute(dbtx.as_mut())
        .await?;

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
CREATE TABLE IF NOT EXISTS supply_initial_genesis (
    value BIGINT PRIMARY KEY,
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
