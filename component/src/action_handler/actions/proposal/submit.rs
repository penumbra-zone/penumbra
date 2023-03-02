use std::sync::Arc;

use anyhow::{Context, Result};
use ark_ff::PrimeField;
use async_trait::async_trait;
use decaf377::Fq;
use once_cell::sync::Lazy;
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

use penumbra_chain::StateReadExt as _;
use penumbra_crypto::{
    keys::{FullViewingKey, NullifierKey},
    rdsa::{VerificationKey, VerificationKeyBytes},
};
use penumbra_crypto::{ProposalNft, VotingReceiptToken, STAKING_TOKEN_DENOM};
use penumbra_storage::{StateDelta, StateRead, StateWrite};
use penumbra_transaction::plan::TransactionPlan;
use penumbra_transaction::proposal::{
    self, Proposal, ProposalPayload, PROPOSAL_DESCRIPTION_LIMIT, PROPOSAL_TITLE_LIMIT,
};
use penumbra_transaction::{action::ProposalSubmit, Transaction};
use penumbra_transaction::{AuthorizationData, WitnessData};

use crate::action_handler::ActionHandler;
use crate::governance::{StateReadExt as _, StateWriteExt as _};
use crate::shielded_pool::{StateReadExt, StateWriteExt as _, SupplyWrite};

#[async_trait]
impl ActionHandler for ProposalSubmit {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        let ProposalSubmit {
            proposal,
            deposit_amount: _, // we don't check the deposit amount because it's defined by state
        } = self;
        let Proposal {
            id: _, // we can't check the ID statelessly because it's defined by state
            title,
            description,
            payload,
        } = proposal;

        if title.len() > PROPOSAL_TITLE_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal title must fit within {PROPOSAL_TITLE_LIMIT} characters"
            ));
        }

        if description.len() > PROPOSAL_DESCRIPTION_LIMIT {
            return Err(anyhow::anyhow!(
                "proposal description must fit within {PROPOSAL_DESCRIPTION_LIMIT} characters"
            ));
        }

        use penumbra_transaction::action::ProposalPayload::*;
        match payload {
            Signaling { commit: _ } => { /* all signaling proposals are valid */ }
            Emergency { halt_chain: _ } => { /* all emergency proposals are valid */ }
            ParameterChange { old, new } => {
                old.check_valid_update(new)
                    .context("invalid change to chain parameters")?;
            }
            DaoSpend { transaction_plan } => {
                // Check to make sure that the transaction plan contains only valid actions for the
                // DAO (none of them should require proving to build):
                use penumbra_transaction::plan::ActionPlan::*;
                for action in &transaction_plan.actions {
                    match action {
                        Spend(_) | Output(_) | Swap(_) | SwapClaim(_) | DelegatorVote(_) => {
                            // These actions all require proving, so they are banned from DAO spend
                            // proposals to prevent DoS attacks.
                            anyhow::bail!(
                                "invalid action in DAO spend proposal (would require proving)"
                            )
                        }
                        ProposalSubmit(_) | ProposalWithdraw(_) | ProposalDepositClaim(_) => {
                            // These actions manipulate proposals, so they are banned from DAO spend
                            // actions because they could cause recursion.
                            anyhow::bail!("invalid action in DAO spend proposal (not allowed to manipulate proposals from within proposals)")
                        }
                        Delegate(_)
                        | Undelegate(_)
                        | UndelegateClaim(_)
                        | ValidatorDefinition(_)
                        | IBCAction(_)
                        | ValidatorVote(_)
                        | PositionOpen(_)
                        | PositionClose(_)
                        | PositionWithdraw(_)
                        | PositionRewardClaim(_)
                        | DaoSpend(_)
                        | DaoOutput(_)
                        | DaoDeposit(_) => {
                            // These actions are all valid for DAO spend proposals, because they
                            // don't require proving, so they don't represent a DoS vector.
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let ProposalSubmit {
            deposit_amount,
            proposal, // statelessly verified
        } = self;

        // Check that the deposit amount agrees with the chain parameters
        let chain_parameters = state.get_chain_params().await?;
        if *deposit_amount != chain_parameters.proposal_deposit_amount {
            anyhow::bail!(
                "submitted proposal deposit of {}{} does not match required proposal deposit of {}{}",
                deposit_amount,
                *STAKING_TOKEN_DENOM,
                chain_parameters.proposal_deposit_amount,
                *STAKING_TOKEN_DENOM,
            );
        }

        // Check that the proposal ID is the correct next proposal ID
        let next_proposal_id = state.next_proposal_id().await?;
        if proposal.id != next_proposal_id {
            anyhow::bail!(
                "submitted proposal ID {} does not match expected proposal ID {}",
                proposal.id,
                next_proposal_id,
            );
        }

        match &proposal.payload {
            ProposalPayload::Signaling { .. } => { /* no stateful checks for signaling */ }
            ProposalPayload::Emergency { .. } => { /* no stateful checks for emergency */ }
            ProposalPayload::ParameterChange { .. } => {
                /* no stateful checks for parameter change (checks are applied when proposal finishes) */
            }
            ProposalPayload::DaoSpend { transaction_plan } => {
                // If DAO spend proposals aren't enabled, then we can't allow them to be submitted
                anyhow::ensure!(
                    chain_parameters.dao_spend_proposals_enabled,
                    "DAO spend proposals are not enabled",
                );

                // Check that the transaction plan can be built without any witness or auth data and
                // it passes stateless and stateful checks, and can be executed successfully in the
                // current chain state. This doesn't guarantee that it will execute successfully at
                // the time when the proposal passes, but we don't want to allow proposals that are
                // obviously going to fail to execute.
                let tx = build_dao_transaction(transaction_plan.clone())
                    .await
                    .context("failed to build submitted DAO spend transaction plan")?;
                tx.check_stateless(Arc::new(tx.clone()))
                    .await
                    .context("submitted DAO spend transaction failed stateless checks")?;
                tx.check_stateful(state.clone())
                    .await
                    .context("submitted DAO spend transaction failed stateful checks")?;
                tx.execute(StateDelta::new(state)).await.context(
                    "submitted DAO spend transaction failed to execute in current chain state",
                )?;
            }
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let ProposalSubmit {
            proposal,
            deposit_amount,
        } = self;

        // If the proposal is a DAO spend proposal, we've already built it, but we need to build it
        // again because we can't remember anything from `check_tx_stateful` to `execute`:
        if let ProposalPayload::DaoSpend { transaction_plan } = &proposal.payload {
            // Build the transaction again (this time we know it will succeed because it built and
            // passed all checks in `check_tx_stateful`):
            let tx = build_dao_transaction(transaction_plan.clone())
                .await
                .context("failed to build submitted DAO spend transaction plan in execute step")?;

            // Cache the built transaction in the state so we can use it later, without rebuilding:
            state.put_dao_transaction(proposal.id, tx);
        }

        // Store the contents of the proposal and generate a fresh proposal id for it
        let proposal_id = state
            .new_proposal(proposal)
            .await
            .context("can create proposal")?;

        // Set the deposit amount for the proposal
        state.put_deposit_amount(proposal_id, *deposit_amount);

        // Register the denom for the voting proposal NFT
        state
            .register_denom(&ProposalNft::deposit(proposal_id).denom())
            .await?;

        // Register the denom for the vote receipt tokens
        state
            .register_denom(&VotingReceiptToken::new(proposal_id).denom())
            .await?;

        // Set the proposal state to voting (votes start immediately)
        state.put_proposal_state(proposal_id, proposal::State::Voting);

        // Determine what block it is currently, and calculate when the proposal should start voting
        // (now!) and finish voting (later...), then write that into the state
        let chain_params = state
            .get_chain_params()
            .await
            .context("can get chain params")?;
        let current_block = state
            .get_block_height()
            .await
            .context("can get block height")?;
        let voting_end = current_block + chain_params.proposal_voting_blocks;
        state.put_proposal_voting_start(proposal_id, current_block);
        state.put_proposal_voting_end(proposal_id, voting_end);

        // Compute the effective starting TCT position for the proposal, by rounding the current
        // position down to the start of the block.
        let Some(sct_position) = state.stub_state_commitment_tree().await.position() else {
            anyhow::bail!("state commitment tree is full");
        };
        // All proposals start are considered to start at the beginning of the block, because this
        // means there are no ordering games to be played within the block in which a proposal begins:
        let proposal_start_position = (sct_position.epoch(), sct_position.block(), 0).into();
        state.put_proposal_voting_start_position(proposal_id, proposal_start_position);

        // Since there was a proposal submitted, ensure we track this so that clients can retain
        // state needed to vote as delegators
        let mut compact_block = state.stub_compact_block();
        compact_block.proposal_started = true;
        state.stub_put_compact_block(compact_block);

        tracing::debug!(proposal = %proposal_id, "created proposal");

        Ok(())
    }
}

/// The full viewing key used to construct transactions made by the Penumbra DAO.
///
/// This full viewing key does not correspond to any known spend key; it is constructed from the
/// hashes of two arbitrary strings.
static DAO_FULL_VIEWING_KEY: Lazy<FullViewingKey> = Lazy::new(|| {
    // We start with two different personalization strings for the hash function:
    let ak_personalization = b"Penumbra_DAO_ak";
    let nk_personalization = b"Penumbra_DAO_nk";

    // We pick two different arbitrary strings to hash:
    let ak_hash_input =
        b"This hash input is used to form the `ak` component of the Penumbra DAO's full viewing key.";
    let nk_hash_input =
        b"This hash input is used to form the `nk` component of the Penumbra DAO's full viewing key.";

    // We hash the two strings using their respective personalizations:
    let ak_hash = blake2b_simd::Params::new()
        .personal(ak_personalization)
        .hash(ak_hash_input);
    let nk_hash = blake2b_simd::Params::new()
        .personal(nk_personalization)
        .hash(nk_hash_input);

    // We construct the `ak` component of the full viewing key from the hash of the first string:
    let ak = VerificationKey::try_from(VerificationKeyBytes::from(
        decaf377::Element::encode_to_curve(&Fq::from_le_bytes_mod_order(ak_hash.as_bytes()))
            .vartime_compress()
            .0,
    ))
    .expect("penumbra DAO FVK's `ak` must be a valid verification key by construction");

    // We construct the `nk` component of the full viewing key from the hash of the second string:
    let nk = NullifierKey(Fq::from_le_bytes_mod_order(nk_hash.as_bytes()));

    // We construct the full viewing key from the `ak` and `nk` components:
    FullViewingKey::from_components(ak, nk)
});

/// The seed used for the random number generator used when constructing transactions made by the
/// DAO.
///
/// It is arbitary, but must be deterministic in order to ensure that every node in the network
/// constructs a byte-for-byte identical transaction.
const DAO_TRANSACTION_RNG_SEED: &[u8; 32] = b"Penumbra DAO's tx build rng seed";

async fn build_dao_transaction(transaction_plan: TransactionPlan) -> Result<Transaction> {
    let effect_hash = transaction_plan.effect_hash(&DAO_FULL_VIEWING_KEY);
    transaction_plan
        .build_concurrent(
            &mut ChaCha20Rng::from_seed(*DAO_TRANSACTION_RNG_SEED),
            &DAO_FULL_VIEWING_KEY,
            AuthorizationData {
                effect_hash,
                spend_auths: Default::default(),
                delegator_vote_auths: Default::default(),
            },
            WitnessData {
                anchor: penumbra_tct::Tree::new().root(),
                state_commitment_proofs: Default::default(),
            },
        )
        .await
}

#[cfg(test)]
mod test {
    /// Ensure that the DAO full viewing key can be constructed and does not panic when referenced.
    #[test]
    fn dao_fvk_can_be_constructed() {
        let _ = *super::DAO_FULL_VIEWING_KEY;
    }
}
