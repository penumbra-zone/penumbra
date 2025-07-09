use anyhow::anyhow;
use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext, Version},
    sqlx, AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_app::{event::EventAppParametersChange, genesis::Content, params::AppParameters};
use penumbra_sdk_asset::asset;
use penumbra_sdk_dex::event::EventLqtPositionVolume;
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_distributions::{event::EventLqtPoolSizeIncrease, DistributionsParameters};
use penumbra_sdk_funding::{
    event::{EventLqtDelegatorReward, EventLqtPositionReward, EventLqtVote},
    FundingParameters,
};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::event::EventDomainType;
use penumbra_sdk_sct::{event::EventEpochRoot, params::SctParameters};
use sqlx::types::BigDecimal;

use crate::parsing::parse_content;

struct Parameters {
    funding: FundingParameters,
    sct: SctParameters,
    distribution: DistributionsParameters,
}

impl From<Content> for Parameters {
    fn from(value: Content) -> Self {
        Self {
            funding: value.funding_content.funding_params,
            sct: value.sct_content.sct_params,
            distribution: value.distributions_content.distributions_params,
        }
    }
}

impl From<AppParameters> for Parameters {
    fn from(value: AppParameters) -> Self {
        Self {
            funding: value.funding_params,
            sct: value.sct_params,
            distribution: value.distributions_params,
        }
    }
}

mod _params {
    use super::*;

    /// Set the params post genesis.
    pub async fn set_initial(
        dbtx: &mut PgTransaction<'_>,
        params: Parameters,
    ) -> anyhow::Result<()> {
        set_epoch(dbtx, 0, params).await
    }

    // This will be used once we integrate the event for parameter changes.
    #[allow(dead_code)]
    /// Set the params for a given epoch.
    pub async fn set_epoch(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        params: Parameters,
    ) -> anyhow::Result<()> {
        let gauge_threshold = params.funding.liquidity_tournament.gauge_threshold;
        let delegator_share = params.funding.liquidity_tournament.delegator_share;
        let epoch_duration = params.sct.epoch_duration;
        let rewards_per_block = params.distribution.liquidity_tournament_incentive_per_block;
        sqlx::query(
            "
            INSERT INTO lqt._params
            VALUES ($1, $2::NUMERIC / 100, $3::NUMERIC / 100, $4, $5)
            ON CONFLICT (epoch)
            DO UPDATE SET
                delegator_share = EXCLUDED.delegator_share,
                gauge_threshold = EXCLUDED.gauge_threshold,
                epoch_duration = EXCLUDED.epoch_duration,
                rewards_per_block = EXCLUDED.rewards_per_block
        ",
        )
        .bind(i64::try_from(epoch)?)
        .bind(i64::try_from(delegator_share.to_percent())?)
        .bind(i64::try_from(gauge_threshold.to_percent())?)
        .bind(i64::try_from(epoch_duration)?)
        .bind(i64::try_from(rewards_per_block)?)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod _meta {
    use super::*;

    pub async fn update_with_batch(
        dbtx: &mut PgTransaction<'_>,
        batch: &EventBatch,
        block_time_s: f64,
    ) -> anyhow::Result<()> {
        let current_height = batch
            .events_by_block()
            .last()
            .map(|x| x.height())
            .ok_or(anyhow!("expected there to be at least one block in events"))?;
        sqlx::query(
            "
            INSERT INTO lqt._meta VALUES (0, $1, $2)
            ON CONFLICT (rowid)
            DO UPDATE SET
                current_height = EXCLUDED.current_height,
                block_time_s = EXCLUDED.block_time_s
        ",
        )
        .bind(i64::try_from(current_height)?)
        .bind(block_time_s)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }
}

mod _epoch_info {
    use super::*;

    pub async fn start_epoch(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        height: u64,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO lqt._epoch_info VALUES ($1, $2, $2, NULL, 0) ON CONFLICT DO NOTHING",
        )
        .bind(i64::try_from(epoch)?)
        .bind(i64::try_from(height)?)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn end_epoch(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        height: u64,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE lqt._epoch_info SET end_block = $2, updated_block = $2 WHERE epoch = $1",
        )
        .bind(i64::try_from(epoch)?)
        .bind(i64::try_from(height)?)
        .execute(dbtx.as_mut())
        .await?;
        Ok(())
    }

    pub async fn set_rewards_for_epoch(
        dbtx: &mut PgTransaction<'_>,
        epoch: u64,
        height: u64,
        amount: Amount,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE lqt._epoch_info SET available_rewards = $2::NUMERIC, updated_block = $3 WHERE epoch = $1")
            .bind(i64::try_from(epoch)?)
            .bind(BigDecimal::from(amount.value()))
            .bind(i64::try_from(height)?)
            .execute(dbtx.as_mut())
            .await?;
        Ok(())
    }

    pub async fn current(dbtx: &mut PgTransaction<'_>) -> anyhow::Result<u64> {
        let out: i32 =
            sqlx::query_scalar("SELECT epoch FROM lqt._epoch_info ORDER BY epoch DESC LIMIT 1")
                .fetch_one(dbtx.as_mut())
                .await?;
        Ok(u64::try_from(out)?)
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
pub struct Lqt {
    block_time_s: f64,
}

impl Lqt {
    pub fn new(block_time_s: f64) -> Self {
        Self { block_time_s }
    }

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
            _epoch_info::end_epoch(dbtx, e.index, event.block_height).await?;
            _epoch_info::start_epoch(dbtx, e.index + 1, event.block_height + 1).await?;
        } else if let Ok(e) = EventLqtPoolSizeIncrease::try_from_event(&event.event) {
            _epoch_info::set_rewards_for_epoch(
                dbtx,
                e.epoch_index,
                event.block_height,
                e.new_total,
            )
            .await?;
        } else if let Ok(e) = EventAppParametersChange::try_from_event(&event.event) {
            let current = _epoch_info::current(dbtx).await?;
            _params::set_epoch(dbtx, current, e.new_parameters.into()).await?;
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
        let content = parse_content(app_state.clone())?;
        _params::set_initial(dbtx, content.into()).await?;
        Ok(())
    }

    fn name(&self) -> String {
        "lqt".to_string()
    }

    fn version(&self) -> Version {
        let hash: [u8; 32] = blake2b_simd::Params::default()
            .personal(b"option_hash")
            .hash_length(32)
            .to_state()
            .update(&self.block_time_s.to_le_bytes())
            .finalize()
            .as_bytes()
            .try_into()
            .expect("Impossible 000-003: expected 32 byte hash");

        Version::with_major(4).with_option_hash(hash)
    }

    async fn reset(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("reset.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    async fn on_startup(&self, dbtx: &mut PgTransaction) -> Result<(), anyhow::Error> {
        for statement in include_str!("schema.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        _epoch_info::start_epoch(dbtx, 1, 1).await?;
        Ok(())
    }

    async fn index_batch(
        &self,
        dbtx: &mut PgTransaction,
        batch: EventBatch,
        _ctx: EventBatchContext,
    ) -> Result<(), anyhow::Error> {
        _meta::update_with_batch(dbtx, &batch, self.block_time_s).await?;
        for event in batch.events() {
            self.index_event(dbtx, event).await?;
        }
        Ok(())
    }
}
