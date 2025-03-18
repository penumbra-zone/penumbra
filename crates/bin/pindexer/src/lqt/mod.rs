use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext, Version},
    sqlx, AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::event::EventLqtPositionVolume;
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_distributions::event::EventLqtPoolSizeIncrease;
use penumbra_sdk_funding::event::{EventLqtDelegatorReward, EventLqtPositionReward, EventLqtVote};
use penumbra_sdk_funding::params::LiquidityTournamentParameters;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::event::EventDomainType;
use penumbra_sdk_sct::event::EventEpochRoot;
use sqlx::types::BigDecimal;

use crate::parsing::parse_content;

mod _params {
    use super::*;

    /// Set the params post genesis.
    pub async fn set_initial(
        dbtx: &mut PgTransaction<'_>,
        params: LiquidityTournamentParameters,
    ) -> anyhow::Result<()> {
        set_epoch(dbtx, 0, params).await
    }

    // This will be used once we integrate the event for parameter changes.
    #[allow(dead_code)]
    /// Set the params for a given epoch.
    pub async fn set_epoch(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        params: LiquidityTournamentParameters,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
            INSERT INTO lqt._params
            VALUES ($1, $2::NUMERIC / 100, $3::NUMERIC / 100)
            ON CONFLICT (epoch)
            DO UPDATE SET
                delegator_share = EXCLUDED.delegator_share,
                gauge_threshold = EXCLUDED.gauge_threshold
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(i64::try_from(params.gauge_threshold.to_percent())?)
        .bind(i64::try_from(params.delegator_share.to_percent())?)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod _finished_epochs {
    use super::*;

    /// Declare that a particular epoch has ended.
    ///
    /// This is idempotent.
    pub async fn declare_finished(dbtx: &mut PgTransaction<'_>, epoch: u64) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO lqt._finished_epochs VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(i64::try_from(epoch)?)
            .execute(dbtx.as_mut())
            .await?;
        Ok(())
    }
}

mod _available_rewards {
    use super::*;

    /// Add some amount to the available rewards for an epoch.
    pub async fn set_for_epoch(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        amount: Amount,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
            INSERT INTO lqt._available_rewards
            VALUES ($1, $2)
            ON CONFLICT (epoch)
            DO UPDATE SET
                amount = EXCLUDED.amount
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(BigDecimal::from(amount.value()))
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod _delegator_rewards {
    use super::*;

    /// Add a reward to a particular delegator in some epoch.
    pub async fn add(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        addr: Address,
        amount: Amount,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
            INSERT INTO lqt._delegator_rewards      
            VALUES ($1, $2, $3)
            ON CONFLICT (epoch, address)
            DO UPDATE SET
                amount = lqt._delegator_rewards.amount + EXCLUDED.amount
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(addr.to_vec())
        .bind(BigDecimal::from(amount.value()))
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod _lp_rewards {
    use super::*;

    /// Add a reward to a given LP in some epoch.
    ///
    /// This assumes that the LP has already been created.
    pub async fn add_reward(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        position_id: position::Id,
        amount: Amount,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
            UPDATE lqt._lp_rewards       
            SET amount = amount + $3
            WHERE epoch = $1 AND position_id = $2
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(position_id.0)
        .bind(BigDecimal::from(amount.value()))
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    /// Add flows to some LP's state.
    pub async fn add_flows(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        position_id: position::Id,
        other_asset: asset::Id,
        um_volume: Amount,
        asset_volume: Amount,
        um_fees: Amount,
        asset_fees: Amount,
        points: Amount,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
            INSERT INTO lqt._lp_rewards      
            VALUES ($1, $2, $3, 0, 1, $4, $5, $6, $7, $8)
            ON CONFLICT (epoch, position_id)
            DO UPDATE SET
                asset_id = EXCLUDED.asset_id,
                executions = lqt._lp_rewards.executions + 1,
                um_volume = lqt._lp_rewards.um_volume + EXCLUDED.um_volume,
                asset_volume = lqt._lp_rewards.asset_volume + EXCLUDED.asset_volume,
                um_fees = lqt._lp_rewards.um_fees + EXCLUDED.um_fees,
                asset_fees = lqt._lp_rewards.asset_fees + EXCLUDED.asset_fees,
                points = lqt._lp_rewards.points + EXCLUDED.points
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(position_id.0)
        .bind(other_asset.to_bytes())
        .bind(BigDecimal::from(um_volume.value()))
        .bind(BigDecimal::from(asset_volume.value()))
        .bind(BigDecimal::from(um_fees.value()))
        .bind(BigDecimal::from(asset_fees.value()))
        .bind(BigDecimal::from(points.value()))
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod _votes {
    use super::*;

    /// Add a vote in a given epoch.
    pub async fn add(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        power: Amount,
        asset_id: asset::Id,
        addr: Address,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "
            INSERT INTO lqt._votes
            VALUES (DEFAULT, $1, $2, $3, $4)      
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(BigDecimal::from(power.value()))
        .bind(asset_id.to_bytes())
        .bind(addr.to_vec())
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Lqt {}

impl Lqt {
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction<'_>,
        event: ContextualizedEvent<'_>,
    ) -> anyhow::Result<()> {
        if let Ok(e) = EventLqtVote::try_from_event(&event.event) {
            _votes::add(
                dbtx,
                e.epoch_index,
                e.voting_power,
                e.incentivized_asset_id,
                e.rewards_recipient,
            )
            .await?;
        } else if let Ok(e) = EventLqtDelegatorReward::try_from_event(&event.event) {
            _delegator_rewards::add(dbtx, e.epoch_index, e.address, e.reward_amount).await?;
        } else if let Ok(e) = EventLqtPositionReward::try_from_event(&event.event) {
            _lp_rewards::add_reward(dbtx, e.epoch_index, e.position_id, e.reward_amount).await?;
        } else if let Ok(e) = EventLqtPositionVolume::try_from_event(&event.event) {
            _lp_rewards::add_flows(
                dbtx,
                e.epoch_index,
                e.position_id,
                e.asset_id,
                e.staking_token_in,
                e.asset_in,
                e.staking_fees,
                e.asset_fees,
                e.volume,
            )
            .await?;
        } else if let Ok(e) = EventEpochRoot::try_from_event(&event.event) {
            _finished_epochs::declare_finished(dbtx, e.index).await?;
        } else if let Ok(e) = EventLqtPoolSizeIncrease::try_from_event(&event.event) {
            _available_rewards::set_for_epoch(dbtx, e.epoch_index, e.new_total).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl AppView for Lqt {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        for statement in include_str!("schema.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        let content = parse_content(app_state.clone())?;
        let params = content.funding_content.funding_params.liquidity_tournament;
        _params::set_initial(dbtx, params).await?;
        Ok(())
    }

    fn name(&self) -> String {
        "lqt".to_string()
    }

    fn version(&self) -> Version {
        Version::with_major(1)
    }

    async fn reset(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("reset.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
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
}
