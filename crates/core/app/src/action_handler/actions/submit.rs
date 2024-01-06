use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use ark_ff::PrimeField;
use async_trait::async_trait;
use cnidarium::{StateDelta, StateRead, StateWrite};
use decaf377::Fq;
use decaf377_rdsa::{VerificationKey, VerificationKeyBytes};
use ibc_types::core::client::ClientId;
use once_cell::sync::Lazy;
use penumbra_asset::STAKING_TOKEN_DENOM;
use penumbra_chain::component::StateReadExt as _;
use penumbra_community_pool::component::StateReadExt as _;
use penumbra_ibc::component::ClientStateReadExt;
use penumbra_keys::keys::{FullViewingKey, NullifierKey};
use penumbra_proto::DomainType;
use penumbra_sct::component::StateReadExt as _;
use penumbra_shielded_pool::component::SupplyWrite;

use penumbra_transaction::plan::TransactionPlan;
use penumbra_transaction::Transaction;
use penumbra_transaction::{AuthorizationData, WitnessData};

use crate::action_handler::ActionHandler;
use crate::community_pool_ext::CommunityPoolStateWriteExt;
use crate::params::AppParameters;
use penumbra_governance::{
    component::{StateReadExt as _, StateWriteExt as _},
    proposal::{Proposal, ProposalPayload},
    proposal_state::State as ProposalState,
    ProposalNft, ProposalSubmit, VotingReceiptToken,
};

// IMPORTANT: these length limits are enforced by consensus! Changing them will change which
// transactions are accepted by the network, and so they *cannot* be changed without a network
// upgrade!

// This is enough room to print "Proposal #999,999: $TITLE" in 99 characters (and the
// proposal title itself in 80), a decent line width for a modern terminal, as well as a
// reasonable length for other interfaces.
pub const PROPOSAL_TITLE_LIMIT: usize = 80; // ⚠️ DON'T CHANGE THIS (see above)!

// Limit the size of a description to 10,000 characters (a reasonable limit borrowed from
// the Cosmos SDK).
pub const PROPOSAL_DESCRIPTION_LIMIT: usize = 10_000; // ⚠️ DON'T CHANGE THIS (see above)!

#[async_trait]
impl ActionHandler for ProposalSubmit {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
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
            anyhow::bail!("proposal title must fit within {PROPOSAL_TITLE_LIMIT} characters");
        }

        if description.len() > PROPOSAL_DESCRIPTION_LIMIT {
            anyhow::bail!(
                "proposal description must fit within {PROPOSAL_DESCRIPTION_LIMIT} characters"
            );
        }

        use penumbra_governance::ProposalPayload::*;
        match payload {
            Signaling { commit: _ } => { /* all signaling proposals are valid */ }
            Emergency { halt_chain: _ } => { /* all emergency proposals are valid */ }
            ParameterChange { old, new } => {
                // Since the changed app parameters is a differential, we need to construct
                // a complete AppParameters:
                //
                // `old_app_params` should be complete and represent the state of all app parameters
                // at the time the proposal was created.
                let old_app_params = AppParameters::from_changed_params(old, None)?;
                // `new_app_params` should be sparse and only the components whose parameters were changed
                // by the proposal should be `Some`.
                let new_app_params =
                    AppParameters::from_changed_params(new, Some(&old_app_params))?;
                old_app_params
                    .check_valid_update(&new_app_params)
                    .context("invalid change to app parameters")?;
            }
            CommunityPoolSpend { transaction_plan } => {
                // Check to make sure that the transaction plan contains only valid actions for the
                // Community Pool (none of them should require proving to build):
                use penumbra_transaction::plan::ActionPlan::*;

                let parsed_transaction_plan = TransactionPlan::decode(&transaction_plan[..])
                    .context("transaction plan was malformed")?;

                for action in &parsed_transaction_plan.actions {
                    match action {
                        Spend(_) | Output(_) | Swap(_) | SwapClaim(_) | DelegatorVote(_)
                        | UndelegateClaim(_) => {
                            // These actions all require proving, so they are banned from Community Pool spend
                            // proposals to prevent DoS attacks.
                            anyhow::bail!(
                                "invalid action in Community Pool spend proposal (would require proving)"
                            )
                        }
                        Delegate(_) | Undelegate(_) => {
                            // Delegation and undelegation is disallowed due to Undelegateclaim requiring proving.
                            anyhow::bail!(
                                "invalid action in Community Pool spend proposal (can't claim outputs of undelegation)"
                            )
                        }
                        ProposalSubmit(_) | ProposalWithdraw(_) | ProposalDepositClaim(_) => {
                            // These actions manipulate proposals, so they are banned from Community Pool spend
                            // actions because they could cause recursion.
                            anyhow::bail!("invalid action in Community Pool spend proposal (not allowed to manipulate proposals from within proposals)")
                        }
                        ValidatorDefinition(_)
                        | IbcAction(_)
                        | ValidatorVote(_)
                        | PositionOpen(_)
                        | PositionClose(_)
                        | PositionWithdraw(_)
                        | PositionRewardClaim(_)
                        | CommunityPoolSpend(_)
                        | CommunityPoolOutput(_)
                        | Withdrawal(_)
                        | CommunityPoolDeposit(_) => {
                            // These actions are all valid for Community Pool spend proposals, because they
                            // don't require proving, so they don't represent a DoS vector.
                        }
                    }
                }
            }
            UpgradePlan { .. } => {}
            FreezeIbcClient { client_id } => {
                let _ = &ClientId::from_str(client_id)
                    .context("can't decode client id from IBC proposal")?;
            }
            UnfreezeIbcClient { client_id } => {
                let _ = &ClientId::from_str(client_id)
                    .context("can't decode client id from IBC proposal")?;
            }
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let ProposalSubmit {
            deposit_amount,
            proposal, // statelessly verified
        } = self;

        // Check that the deposit amount agrees with the parameters
        let governance_parameters = state.get_governance_params().await?;
        if *deposit_amount != governance_parameters.proposal_deposit_amount {
            anyhow::bail!(
                "submitted proposal deposit of {}{} does not match required proposal deposit of {}{}",
                deposit_amount,
                *STAKING_TOKEN_DENOM,
                governance_parameters.proposal_deposit_amount,
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
            ProposalPayload::CommunityPoolSpend { transaction_plan } => {
                // If Community Pool spend proposals aren't enabled, then we can't allow them to be submitted
                let community_pool_parameters = state.get_community_pool_params().await?;
                anyhow::ensure!(
                    community_pool_parameters.community_pool_spend_proposals_enabled,
                    "Community Pool spend proposals are not enabled",
                );

                // Check that the transaction plan can be built without any witness or auth data and
                // it passes stateless and stateful checks, and can be executed successfully in the
                // current chain state. This doesn't guarantee that it will execute successfully at
                // the time when the proposal passes, but we don't want to allow proposals that are
                // obviously going to fail to execute.
                let parsed_transaction_plan = TransactionPlan::decode(&transaction_plan[..])
                    .context("transaction plan was malformed")?;
                let tx = build_community_pool_transaction(parsed_transaction_plan.clone())
                    .await
                    .context("failed to build submitted Community Pool spend transaction plan")?;
                tx.check_stateless(()).await.context(
                    "submitted Community Pool spend transaction failed stateless checks",
                )?;
                tx.check_stateful(state.clone())
                    .await
                    .context("submitted Community Pool spend transaction failed stateful checks")?;
                tx.execute(StateDelta::new(state)).await.context(
                    "submitted Community Pool spend transaction failed to execute in current chain state",
                )?;
            }
            ProposalPayload::UpgradePlan { .. } => {
                // TODO(erwan): no stateful checks for upgrade plan.
            }
            ProposalPayload::FreezeIbcClient { client_id } => {
                // Check that the client ID is valid and that there is a corresponding
                // client state. If the client state is already frozen, then freezing it
                // is a no-op.
                let client_id = &ClientId::from_str(client_id)
                    .map_err(|e| tonic::Status::aborted(format!("invalid client id: {e}")))?;
                let _ = state.get_client_state(client_id).await?;
            }
            ProposalPayload::UnfreezeIbcClient { client_id } => {
                // Check that the client ID is valid and that there is a corresponding
                // client state. If the client state is not frozen, then unfreezing it
                // is a no-op.
                let client_id = &ClientId::from_str(client_id)
                    .map_err(|e| tonic::Status::aborted(format!("invalid client id: {e}")))?;
                let _ = state.get_client_state(client_id).await?;
            }
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let ProposalSubmit {
            proposal,
            deposit_amount,
        } = self;

        // If the proposal is a Community Pool spend proposal, we've already built it, but we need to build it
        // again because we can't remember anything from `check_tx_stateful` to `execute`:
        if let ProposalPayload::CommunityPoolSpend { transaction_plan } = &proposal.payload {
            // Build the transaction again (this time we know it will succeed because it built and
            // passed all checks in `check_tx_stateful`):
            let parsed_transaction_plan = TransactionPlan::decode(&transaction_plan[..])
                .context("transaction plan was malformed")?;
            let tx = build_community_pool_transaction(parsed_transaction_plan.clone())
                .await
                .context("failed to build submitted Community Pool spend transaction plan in execute step")?;

            // Cache the built transaction in the state so we can use it later, without rebuilding:
            state.put_community_pool_transaction(proposal.id, tx);
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
        state.put_proposal_state(proposal_id, ProposalState::Voting);

        // Determine what block it is currently, and calculate when the proposal should start voting
        // (now!) and finish voting (later...), then write that into the state
        let governance_params = state
            .get_governance_params()
            .await
            .context("can get chain params")?;
        let current_block = state
            .get_block_height()
            .await
            .context("can get block height")?;
        let voting_end = current_block + governance_params.proposal_voting_blocks;
        state.put_proposal_voting_start(proposal_id, current_block);
        state.put_proposal_voting_end(proposal_id, voting_end);

        // Compute the effective starting TCT position for the proposal, by rounding the current
        // position down to the start of the block.
        let Some(sct_position) = state.state_commitment_tree().await.position() else {
            anyhow::bail!("state commitment tree is full");
        };
        // All proposals start are considered to start at the beginning of the block, because this
        // means there are no ordering games to be played within the block in which a proposal begins:
        let proposal_start_position = (sct_position.epoch(), sct_position.block(), 0).into();
        state.put_proposal_voting_start_position(proposal_id, proposal_start_position);

        // Since there was a proposal submitted, ensure we track this so that clients can retain
        // state needed to vote as delegators
        state.mark_proposal_started();

        tracing::debug!(proposal = %proposal_id, "created proposal");

        Ok(())
    }
}

/// The full viewing key used to construct transactions made by the Penumbra Community Pool.
///
/// This full viewing key does not correspond to any known spend key; it is constructed from the
/// hashes of two arbitrary strings.
static COMMUNITY_POOL_FULL_VIEWING_KEY: Lazy<FullViewingKey> = Lazy::new(|| {
    // We start with two different personalization strings for the hash function:
    let ak_personalization = b"Penumbra_CP_ak";
    let nk_personalization = b"Penumbra_CP_nk";

    // We pick two different arbitrary strings to hash:
    let ak_hash_input =
        b"This hash input is used to form the `ak` component of the Penumbra Community Pool's full viewing key.";
    let nk_hash_input =
        b"This hash input is used to form the `nk` component of the Penumbra Community Pool's full viewing key.";

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
    .expect("penumbra Community Pool FVK's `ak` must be a valid verification key by construction");

    // We construct the `nk` component of the full viewing key from the hash of the second string:
    let nk = NullifierKey(Fq::from_le_bytes_mod_order(nk_hash.as_bytes()));

    // We construct the full viewing key from the `ak` and `nk` components:
    FullViewingKey::from_components(ak, nk)
});

async fn build_community_pool_transaction(
    transaction_plan: TransactionPlan,
) -> Result<Transaction> {
    let effect_hash = transaction_plan.effect_hash(&COMMUNITY_POOL_FULL_VIEWING_KEY)?;
    transaction_plan.build(
        &COMMUNITY_POOL_FULL_VIEWING_KEY,
        &WitnessData {
            anchor: penumbra_tct::Tree::new().root(),
            state_commitment_proofs: Default::default(),
        },
        &AuthorizationData {
            effect_hash,
            spend_auths: Default::default(),
            delegator_vote_auths: Default::default(),
        },
    )
}

#[cfg(test)]
mod test {
    /// Ensure that the Community Pool full viewing key can be constructed and does not panic when referenced.
    #[test]
    fn community_pool_fvk_can_be_constructed() {
        let _ = *super::COMMUNITY_POOL_FULL_VIEWING_KEY;
    }
}
