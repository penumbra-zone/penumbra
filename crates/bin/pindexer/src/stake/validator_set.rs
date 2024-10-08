use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgPool, PgTransaction};

use penumbra_app::genesis::Content;
use penumbra_asset::asset;
use penumbra_num::Amount;
use penumbra_proto::{core::component::stake::v1 as pb, event::ProtoEvent};
use penumbra_stake::{
    validator::{self, Validator},
    IdentityKey,
};

use crate::parsing::parse_content;

#[derive(Debug)]
pub struct ValidatorSet {}

#[async_trait]
impl AppView for ValidatorSet {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            // table name is module path + struct name
            // note: protobuf data is encoded as protojson for ease of consumers
            // hence TEXT fields
            "CREATE TABLE stake_validator_set (
                id SERIAL PRIMARY KEY,
                ik TEXT NOT NULL,
                name TEXT NOT NULL,
                definition TEXT NOT NULL,
                voting_power BIGINT NOT NULL,
                queued_delegations BIGINT NOT NULL,
                queued_undelegations BIGINT NOT NULL,
                validator_state TEXT NOT NULL,
                bonding_state TEXT NOT NULL
            );",
        )
        .execute(dbtx.as_mut())
        .await?;

        sqlx::query("CREATE UNIQUE INDEX idx_stake_validator_set_ik ON stake_validator_set(ik);")
            .execute(dbtx.as_mut())
            .await?;

        add_genesis_validators(dbtx, &parse_content(app_state.clone())?).await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        match type_str {
            "penumbra.core.component.stake.v1.EventValidatorDefinitionUpload" => true,
            "penumbra.core.component.stake.v1.EventDelegate" => true,
            "penumbra.core.component.stake.v1.EventUndelegate" => true,
            "penumbra.core.component.stake.v1.EventValidatorVotingPowerChange" => true,
            "penumbra.core.component.stake.v1.EventValidatorStateChange" => true,
            "penumbra.core.component.stake.v1.EventValidatorBondingStateChange" => true,
            _ => false,
        }
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        match event.event.kind.as_str() {
            "penumbra.core.component.stake.v1.EventValidatorDefinitionUpload" => {
                let pe = pb::EventValidatorDefinitionUpload::from_event(event.as_ref())?;
                let val = Validator::try_from(
                    pe.validator
                        .ok_or_else(|| anyhow!("missing validator in event"))?,
                )?;

                handle_upload(dbtx, val).await?;
            }
            "penumbra.core.component.stake.v1.EventDelegate" => {
                let pe = pb::EventDelegate::from_event(event.as_ref())?;
                let ik = IdentityKey::try_from(
                    pe.identity_key
                        .ok_or_else(|| anyhow!("missing ik in event"))?,
                )?;
                let amount = Amount::try_from(
                    pe.amount
                        .ok_or_else(|| anyhow!("missing amount in event"))?,
                )?;

                handle_delegate(dbtx, ik, amount).await?;
            }
            "penumbra.core.component.stake.v1.EventUndelegate" => {
                let pe = pb::EventUndelegate::from_event(event.as_ref())?;
                let ik = IdentityKey::try_from(
                    pe.identity_key
                        .ok_or_else(|| anyhow!("missing ik in event"))?,
                )?;
                let amount = Amount::try_from(
                    pe.amount
                        .ok_or_else(|| anyhow!("missing amount in event"))?,
                )?;
                handle_undelegate(dbtx, ik, amount).await?;
            }
            "penumbra.core.component.stake.v1.EventValidatorVotingPowerChange" => {
                let pe = pb::EventValidatorVotingPowerChange::from_event(event.as_ref())?;
                let ik = IdentityKey::try_from(
                    pe.identity_key
                        .ok_or_else(|| anyhow!("missing ik in event"))?,
                )?;
                let voting_power = Amount::try_from(
                    pe.voting_power
                        .ok_or_else(|| anyhow!("missing amount in event"))?,
                )?;
                handle_voting_power_change(dbtx, ik, voting_power).await?;
            }
            "penumbra.core.component.stake.v1.EventValidatorStateChange" => {
                let pe = pb::EventValidatorStateChange::from_event(event.as_ref())?;
                let ik = IdentityKey::try_from(
                    pe.identity_key
                        .ok_or_else(|| anyhow!("missing ik in event"))?,
                )?;
                let state = validator::State::try_from(
                    pe.state.ok_or_else(|| anyhow!("missing state in event"))?,
                )?;
                handle_validator_state_change(dbtx, ik, state).await?;
            }
            "penumbra.core.component.stake.v1.EventValidatorBondingStateChange" => {
                let pe = pb::EventValidatorBondingStateChange::from_event(event.as_ref())?;
                let ik = IdentityKey::try_from(
                    pe.identity_key
                        .ok_or_else(|| anyhow!("missing ik in event"))?,
                )?;
                let bonding_state = validator::BondingState::try_from(
                    pe.bonding_state
                        .ok_or_else(|| anyhow!("missing bonding_state in event"))?,
                )?;
                handle_validator_bonding_state_change(dbtx, ik, bonding_state).await?;
            }
            _ => {}
        }

        Ok(())
    }
}

async fn add_genesis_validators<'a>(dbtx: &mut PgTransaction<'a>, content: &Content) -> Result<()> {
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

    for val in &content.stake_content.validators {
        // FIXME: this shouldn't be a proto type but now that has been propagated
        // all through the rest of the code for no reason
        let val = Validator::try_from(val.clone())?;
        let delegation_amount = allos.get(&val.token().id()).cloned().unwrap_or_default();

        // insert sql
        sqlx::query(
            "INSERT INTO stake_validator_set (
                ik, name, definition, voting_power, queued_delegations, 
                queued_undelegations, validator_state, bonding_state
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(val.identity_key.to_string())
        .bind(val.name.clone())
        .bind(serde_json::to_string(&val).expect("can serialize"))
        .bind(delegation_amount.value() as i64)
        .bind(0) // queued_delegations
        .bind(0) // queued_undelegations
        .bind(serde_json::to_string(&validator::State::Active).unwrap()) // see add_genesis_validator
        .bind(serde_json::to_string(&validator::BondingState::Bonded).unwrap()) // see add_genesis_validator
        .execute(dbtx.as_mut())
        .await?;
    }

    Ok(())
}

async fn handle_upload<'a>(dbtx: &mut PgTransaction<'a>, val: Validator) -> Result<()> {
    // First, check if the validator already exists
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM stake_validator_set WHERE ik = $1)")
            .bind(&val.identity_key.to_string())
            .fetch_one(dbtx.as_mut())
            .await?;

    if exists {
        // Update existing validator, leaving all the other data like state, VP etc unchanged
        sqlx::query(
            "UPDATE stake_validator_set SET
                name = $2,
                definition = $3
            WHERE ik = $1",
        )
        .bind(val.identity_key.to_string())
        .bind(val.name.clone())
        .bind(serde_json::to_string(&val).expect("can serialize"))
        .execute(dbtx.as_mut())
        .await?;
    } else {
        // Insert new validator
        sqlx::query(
            "INSERT INTO stake_validator_set (
                ik, name, definition, voting_power, queued_delegations, 
                queued_undelegations, validator_state, bonding_state
            )
            VALUES ($1, $2, $3, 0, 0, 0, $4, $5)",
        )
        .bind(val.identity_key.to_string())
        .bind(val.name.clone())
        .bind(serde_json::to_string(&val).expect("can serialize"))
        .bind(serde_json::to_string(&validator::State::Defined).expect("can serialize")) // ValidatorManager::add_validator
        .bind(serde_json::to_string(&validator::BondingState::Unbonded).expect("can serialize")) // ValidatorManager::add_validator
        .execute(dbtx.as_mut())
        .await?;
    }

    Ok(())
}

async fn handle_delegate<'a>(
    dbtx: &mut PgTransaction<'a>,
    ik: IdentityKey,
    amount: Amount,
) -> Result<()> {
    // Update the validator's voting power and queued delegations
    let rows_affected = sqlx::query(
        "UPDATE stake_validator_set 
        SET 
            queued_delegations = queued_delegations + $2
        WHERE ik = $1",
    )
    .bind(ik.to_string())
    .bind(amount.value() as i64)
    .execute(dbtx.as_mut())
    .await?
    .rows_affected();

    // Check if the update was successful
    if rows_affected == 0 {
        anyhow::bail!("No validator found with the given identity key");
    }

    Ok(())
}

async fn handle_undelegate<'a>(
    dbtx: &mut PgTransaction<'a>,
    ik: IdentityKey,
    amount: Amount,
) -> Result<()> {
    // Update only the queued undelegations
    let rows_affected = sqlx::query(
        "UPDATE stake_validator_set 
        SET 
            queued_undelegations = queued_undelegations + $2
        WHERE ik = $1",
    )
    .bind(ik.to_string())
    .bind(amount.value() as i64)
    .execute(dbtx.as_mut())
    .await?
    .rows_affected();

    // Check if the update was successful
    if rows_affected == 0 {
        anyhow::bail!("No validator found with the given identity key");
    }

    Ok(())
}

async fn handle_voting_power_change<'a>(
    dbtx: &mut PgTransaction<'a>,
    ik: IdentityKey,
    voting_power: Amount,
) -> Result<()> {
    // Update the validator's voting power and reset queued delegations/undelegations
    let rows_affected = sqlx::query(
        "UPDATE stake_validator_set 
        SET 
            voting_power = $2, 
            queued_delegations = 0,
            queued_undelegations = 0
        WHERE ik = $1",
    )
    .bind(ik.to_string())
    .bind(voting_power.value() as i64)
    .execute(dbtx.as_mut())
    .await?
    .rows_affected();

    // Check if the update was successful
    if rows_affected == 0 {
        anyhow::bail!("No validator found with the given identity key");
    }

    Ok(())
}

async fn handle_validator_state_change<'a>(
    dbtx: &mut PgTransaction<'a>,
    ik: IdentityKey,
    state: validator::State,
) -> Result<()> {
    // Update the validator's state
    let rows_affected = sqlx::query(
        "UPDATE stake_validator_set 
        SET 
            validator_state = $2
        WHERE ik = $1",
    )
    .bind(ik.to_string())
    .bind(serde_json::to_string(&state).expect("can serialize"))
    .execute(dbtx.as_mut())
    .await?
    .rows_affected();

    // Check if the update was successful
    if rows_affected == 0 {
        anyhow::bail!("No validator found with the given identity key");
    }

    Ok(())
}

async fn handle_validator_bonding_state_change<'a>(
    dbtx: &mut PgTransaction<'a>,
    ik: IdentityKey,
    bonding_state: validator::BondingState,
) -> Result<()> {
    // Update the validator's bonding state
    let rows_affected = sqlx::query(
        "UPDATE stake_validator_set 
        SET 
            bonding_state = $2
        WHERE ik = $1",
    )
    .bind(ik.to_string())
    .bind(serde_json::to_string(&bonding_state).expect("can serialize"))
    .execute(dbtx.as_mut())
    .await?
    .rows_affected();

    // Check if the update was successful
    if rows_affected == 0 {
        anyhow::bail!("No validator found with the given identity key");
    }

    Ok(())
}
