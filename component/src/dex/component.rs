use std::collections::BTreeMap;

use crate::shielded_pool::View as _;
use crate::{Component, Context};
use anyhow::{Context as _, Result};
use ark_ff::Zero;
use async_trait::async_trait;
use decaf377::Fr;
use penumbra_chain::{genesis, View as _};
use penumbra_crypto::{
    dex::{BatchSwapOutputData, TradingPair},
    MockFlowCiphertext, SwapFlow, Value, STAKING_TOKEN_ASSET_ID,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::action::swap_claim::ClaimedSwap;
use penumbra_transaction::{action::swap_claim::List as SwapClaimBodyList, Action, Transaction};
use tendermint::abci;
use tracing::instrument;

use super::state_key;

pub struct Dex {
    state: State,
    // Represents swaps taking place in the current block.
    swaps: BTreeMap<TradingPair, SwapFlow>,
    // Represents swaps that have been claimed in the current block.
    claims: Vec<ClaimedSwap>,
}

impl Dex {
    #[instrument(name = "dex", skip(state))]
    pub async fn new(state: State) -> Self {
        Self {
            state,
            swaps: Default::default(),
            claims: Default::default(),
        }
    }
}

#[async_trait]
impl Component for Dex {
    #[instrument(name = "dex", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "dex", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "dex", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        for action in tx.transaction_body.actions.iter() {
            match action {
                Action::PositionOpen { .. }
                | Action::PositionClose { .. }
                | Action::PositionWithdraw { .. }
                | Action::PositionRewardClaim { .. } => {
                    return Err(anyhow::anyhow!("lp actions not supported yet"));
                }
                Action::Swap(swap) => {
                    // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
                    let auth_hash = tx.transaction_body().auth_hash();

                    // 1. Check binding signature.
                    anyhow::Context::context(
                        tx.binding_verification_key()
                            .verify(auth_hash.as_ref(), tx.binding_sig()),
                        "binding signature failed to verify",
                    )?;

                    // 2. Check swap proof
                    swap.proof
                        .verify(
                            // TODO: no value commitments until flow encryption is available
                            // so we pass placeholder values here, the proof doesn't check these right now
                            // and will fail when checking is re-enabled.
                            Value {
                                amount: 0,
                                asset_id: *STAKING_TOKEN_ASSET_ID,
                            }
                            .commit(Fr::zero()),
                            Value {
                                amount: 0,
                                asset_id: *STAKING_TOKEN_ASSET_ID,
                            }
                            .commit(Fr::zero()),
                            swap.body.fee_commitment,
                            swap.body.swap_nft.note_commitment,
                            swap.body.swap_nft.ephemeral_key,
                            swap.body.fee_blinding,
                        )
                        .context("A swap proof did not verify")?;

                    // TODO: are any other checks necessary?

                    return Ok(());
                }
                Action::SwapClaim(swap_claim) => {
                    // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?
                    let auth_hash = tx.transaction_body().auth_hash();

                    // 1. Check binding signature.
                    anyhow::Context::context(
                        tx.binding_verification_key()
                            .verify(auth_hash.as_ref(), tx.binding_sig()),
                        "binding signature failed to verify",
                    )?;

                    let fee = swap_claim.body.fee.clone();

                    // 2. Check swap claim proof
                    let anchor = tx.anchor;
                    swap_claim
                        .proof
                        .verify(
                            anchor,
                            swap_claim.body.nullifier,
                            swap_claim.body.output_data,
                            swap_claim.body.epoch_duration,
                            swap_claim.body.output_1.note_commitment,
                            swap_claim.body.output_2.note_commitment,
                            fee,
                        )
                        .context("a swap claim proof did not verify")?;

                    // TODO: any other stateless checks?

                    return Ok(());
                }
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "dex", skip(self, _ctx, tx))]
    async fn check_tx_stateful(&self, _ctx: Context, tx: &Transaction) -> Result<()> {
        // It's important to reject all LP actions for now, to prevent
        // inflation / minting bugs until we implement all required checks
        // (e.g., minting tokens by withdrawing reserves we don't check)
        for action in tx.transaction_body.actions.iter() {
            match action {
                Action::PositionOpen { .. }
                | Action::PositionClose { .. }
                | Action::PositionWithdraw { .. }
                | Action::PositionRewardClaim { .. } => {
                    return Err(anyhow::anyhow!("lp actions not supported yet"));
                }
                Action::Swap(_swap) => {
                    // TODO: are any other checks necessary?

                    return Ok(());
                }
                Action::SwapClaim(swap_claim) => {
                    // 1. Validate the epoch duration passed in the swap claim matches
                    // what we know.
                    let epoch_duration = self.state.get_epoch_duration().await?;
                    let provided_epoch_duration = swap_claim.body.epoch_duration;
                    if epoch_duration != provided_epoch_duration {
                        return Err(anyhow::anyhow!(
                            "provided epoch duration does not match chain epoch duration"
                        ));
                    }

                    // 2. The stateful check *must* validate that the clearing
                    // prices used in the proof are valid.
                    let provided_output_height = swap_claim.body.output_data.height;
                    let provided_trading_pair = swap_claim.body.output_data.trading_pair;
                    let output_data = self
                        .state
                        .output_data(provided_output_height, provided_trading_pair)
                        .await?
                        // This check also ensures that the height for the swap is in the past, otherwise
                        // the output data would not be present in the JMT.
                        .ok_or_else(|| anyhow::anyhow!("output data not found"))?;

                    if output_data != swap_claim.body.output_data {
                        return Err(anyhow::anyhow!(
                            "provided output data does not match chain output data"
                        ));
                    }

                    return Ok(());
                }
                _ => {}
            }
        }
        Ok(())
    }

    #[instrument(name = "dex", skip(self, _ctx, tx))]
    async fn execute_tx(&mut self, _ctx: Context, tx: &Transaction) {
        for action in tx.transaction_body.actions.iter() {
            match action {
                Action::PositionOpen { .. }
                | Action::PositionClose { .. }
                | Action::PositionWithdraw { .. }
                | Action::PositionRewardClaim { .. } => {}
                Action::Swap(swap) => {
                    // All swaps will be tallied for the block so the
                    // BatchSwapOutputData for the trading pair/block height can
                    // be set during `end_block`.
                    let mut swap_flows = self
                        .swaps
                        .get_mut(&swap.body.trading_pair)
                        .cloned()
                        .unwrap_or_default();

                    // Add the amount of each asset being swapped to the batch swap flow.
                    swap_flows.0 += MockFlowCiphertext::new(swap.body.delta_1);
                    swap_flows.1 += MockFlowCiphertext::new(swap.body.delta_2);

                    // Set the batch swap flow for the trading pair.
                    self.swaps.insert(swap.body.trading_pair, swap_flows);
                }
                Action::SwapClaim(swap_claim) => {
                    // Each swap claim gets their portion of the swap based on their contribution.
                    self.claims
                        .push(ClaimedSwap(swap_claim.body.clone(), tx.id()));
                }
                _ => {}
            }
        }
    }

    #[instrument(name = "dex", skip(self, _ctx, end_block))]
    async fn end_block(&mut self, _ctx: Context, end_block: &abci::request::EndBlock) {
        // For each batch swap during the block, calculate clearing prices and set in the JMT.
        // TODO: since there are no liquidity providers right now, we'll consider all
        // batch swaps to fail
        for (trading_pair, swap_flows) in self.swaps.iter() {
            let (delta_1, delta_2) = (swap_flows.0.mock_decrypt(), swap_flows.1.mock_decrypt());
            let lambda_1 = 0;
            let lambda_2 = 0;
            let output_data = BatchSwapOutputData {
                height: end_block.height.try_into().unwrap(),
                trading_pair: *trading_pair,
                delta_1,
                delta_2,
                lambda_1,
                lambda_2,
                success: false,
            };
            self.state.set_output_data(output_data).await;
        }

        // Tell the shielded pool component to include the claimed output notes in the NCT.
        self.state
            .set_claimed_swap_outputs(
                self.state.get_block_height().await.unwrap(),
                SwapClaimBodyList(self.claims.clone()),
            )
            .await;
    }
}

/// Extension trait providing read/write access to dex data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: StateExt {
    async fn output_data(
        &self,
        height: u64,
        trading_pair: TradingPair,
    ) -> Result<Option<BatchSwapOutputData>> {
        self.get_domain(state_key::output_data(height, trading_pair).into())
            .await
    }

    async fn set_output_data(&mut self, output_data: BatchSwapOutputData) {
        let height = output_data.height;
        let trading_pair = output_data.trading_pair;
        self.put_domain(
            state_key::output_data(height, trading_pair).into(),
            output_data,
        )
        .await;
    }
}

impl<T: StateExt + Send + Sync> View for T {}
