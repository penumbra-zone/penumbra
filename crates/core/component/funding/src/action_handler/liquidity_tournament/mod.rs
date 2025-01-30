use anyhow::Context as _;
use async_trait::async_trait;
use cnidarium::StateWrite;
use penumbra_sdk_asset::asset::Denom;
use penumbra_sdk_proof_params::DELEGATOR_VOTE_PROOF_VERIFICATION_KEY;
use penumbra_sdk_txhash::TransactionContext;

use crate::liquidity_tournament::{
    proof::LiquidityTournamentVoteProofPublic, ActionLiquidityTournamentVote,
    LiquidityTournamentVoteBody, LIQUIDITY_TOURNAMENT_VOTE_DENOM_MAX_BYTES,
};
use cnidarium_component::ActionHandler;

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

    async fn check_and_execute<S: StateWrite>(&self, _state: S) -> anyhow::Result<()> {
        todo!()
    }
}
