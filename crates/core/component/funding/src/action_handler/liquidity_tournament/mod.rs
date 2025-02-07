use anyhow::{anyhow, ensure, Context as _};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_asset::asset::Denom;
use penumbra_sdk_governance::StateReadExt as _;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proof_params::DELEGATOR_VOTE_PROOF_VERIFICATION_KEY;
use penumbra_sdk_proto::{DomainType, StateWriteProto};
use penumbra_sdk_sct::component::{clock::EpochRead as _, source::SourceContext as _};
use penumbra_sdk_sct::epoch::Epoch;
use penumbra_sdk_stake::component::validator_handler::ValidatorDataRead as _;
use penumbra_sdk_stake::validator::State;
use penumbra_sdk_tct::Position;
use penumbra_sdk_txhash::TransactionContext;

use crate::component::liquidity_tournament::{
    nullifier::{NullifierRead as _, NullifierWrite as _},
    votes::StateWriteExt as _,
};
use crate::event;
use crate::liquidity_tournament::{
    proof::LiquidityTournamentVoteProofPublic, ActionLiquidityTournamentVote,
    LiquidityTournamentVoteBody, LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES,
};

fn is_valid_denom(denom: &Denom) -> anyhow::Result<()> {
    anyhow::ensure!(
        denom.denom.len() <= LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES,
        "denom {} is not <= (MAX OF) {}",
        &denom.denom,
        LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES
    );
    anyhow::ensure!(
        denom.denom.starts_with("transfer/"),
        "denom {} is not an IBC transfer asset",
        &denom.denom
    );
    Ok(())
}

// Check that the start position is early enough to vote in the current epoch.
async fn start_position_good_for_epoch(current: Epoch, start: Position) -> anyhow::Result<()> {
    let ok = match (u64::from(start.epoch()), start.block(), start.commitment()) {
        // Anything before now is definitely fine.
        (e, _, _) if e < current.index => true,
        // The ZK proof will check that note being spent is *strictly* earlier than this position,
        // so we can allow the position to be the very first one in this epoch, and this will not
        // allow anything in this epoch to be used *even* if the first created note this epoch
        // is a delegated token.
        (e, 0, 0) if e == current.index => true,
        _ => false,
    };
    anyhow::ensure!(ok, "position {start:?} is not before epoch {current:?}");
    Ok(())
}

// This isolates the logic for how we should handle out of bounds amounts.
fn voting_power(amount: Amount) -> u64 {
    amount
        .value()
        .try_into()
        .expect("someone acquired {amount:?} > u64::MAX worth of delegation tokens!")
}

#[async_trait]
impl ActionHandler for ActionLiquidityTournamentVote {
    type CheckStatelessContext = TransactionContext;

    async fn check_stateless(&self, context: TransactionContext) -> anyhow::Result<()> {
        let Self {
            auth_sig,
            proof,
            body:
                LiquidityTournamentVoteBody {
                    start_position,
                    nullifier,
                    rk,
                    value,
                    incentivized,
                    ..
                },
        } = self;
        // 1. Is it ok to vote on this denom?
        is_valid_denom(incentivized)?;
        // 2. Check spend auth signature using provided spend auth key.
        rk.verify(context.effect_hash.as_ref(), auth_sig)
            .with_context(|| {
                format!(
                    "{} auth signature failed to verify",
                    std::any::type_name::<Self>()
                )
            })?;

        // 3. Verify the proof against the provided anchor and start position:
        let public = LiquidityTournamentVoteProofPublic {
            anchor: context.anchor,
            value: *value,
            nullifier: *nullifier,
            rk: *rk,
            start_position: *start_position,
        };
        proof
            .verify(&DELEGATOR_VOTE_PROOF_VERIFICATION_KEY, public)
            .context("a LiquidityTournamentVote proof did not verify")?;

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> anyhow::Result<()> {
        // 1. Check that the start position can vote in this round.
        let current_epoch = state
            .get_current_epoch()
            .await
            .expect("failed to fetch current epoch");
        start_position_good_for_epoch(current_epoch, self.body.start_position).await?;
        // 2. We can tally, as long as the nullifier hasn't been used yet (this round).
        let nullifier = self.body.nullifier;
        let nullifier_exists = state.get_lqt_spent_nullifier(nullifier).await.is_some();
        anyhow::ensure!(
            !nullifier_exists,
            "nullifier {} already voted in epoch {}",
            self.body.nullifier,
            current_epoch.index
        );
        let tx_id = state
            .get_current_source()
            .expect("source transaction id should be set");
        state.put_lqt_spent_nullifier(current_epoch.index, nullifier, tx_id);
        // 3. Validate that the delegation asset is for a known validator.
        let validator = state
            .validator_by_delegation_asset(self.body.value.asset_id)
            .await?;
        // The ZK proof asserts that we own the delegation notes, so no further checks are needed
        // for the IK.

        // 4. Check that the validator state is not `Defined` or `Tombstoned`
        let Some(validator_state) = state.get_validator_state(&validator).await? else {
            anyhow::bail!("validator {} is unknown", validator)
        };

        ensure!(
            !matches!(validator_state, State::Defined | State::Tombstoned),
            "validator {} is not in a valid state (Defined or Tombstoned)",
            validator
        );

        // 5. Check that the validator rate exists and that the unbonded amount is non-zero.
        let validator_rate = state
            .get_validator_rate(&validator)
            .await?
            .ok_or_else(|| anyhow!("{} has no rate data", validator))?;
        let unbonded_amount = validator_rate.unbonded_amount(self.body.value.amount);

        ensure!(
            unbonded_amount > Amount::zero(),
            "unbonded amount of delegation token is zero"
        );

        // 6. Ok, actually tally.
        let power = voting_power(unbonded_amount);
        let incentivized = self
            .body
            .incentivized_id()
            .ok_or_else(|| anyhow!("{:?} is not a base denom", self.body.incentivized))?;

        // We emit a little event
        state.record_proto(
            event::EventLqtVote {
                epoch_index: current_epoch.index,
                voting_power: power.into(),
                incentivized_asset_id: incentivized,
                incentivized: self.body.incentivized.clone(),
                rewards_recipient: self.body.rewards_recipient.clone(),
                tx_id,
            }
            .to_proto(),
        );

        state
            .tally(
                current_epoch.index,
                incentivized,
                power,
                validator,
                &self.body.rewards_recipient,
            )
            .await;
        Ok(())
    }
}
