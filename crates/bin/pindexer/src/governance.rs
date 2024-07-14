use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_governance::{
    proposal::ProposalPayloadToml, DelegatorVote, Proposal, ProposalDepositClaim, ProposalWithdraw,
    ValidatorVote, Vote,
};
use penumbra_proto::{
    core::component::{governance::v1 as pb, sct::v1 as sct_pb},
    event::ProtoEvent,
};
use penumbra_stake::IdentityKey;

#[derive(Debug)]
pub struct GovernanceProposals {}

#[async_trait]
impl AppView for GovernanceProposals {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(include_str!("governance/schema.sql"))
            .execute(dbtx.as_mut())
            .await?;

        // TODO: If there are any governance-related genesis data, handle it here
        // let app_state: penumbra_app::genesis::AppState =
        //     serde_json::from_value(app_state.clone()).context("error decoding app_state json")?;
        // add_genesis_proposals(dbtx, &app_state).await?;

        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        matches!(
            type_str,
            "penumbra.core.component.governance.v1.EventProposalSubmit"
                | "penumbra.core.component.governance.v1.EventDelegatorVote"
                | "penumbra.core.component.governance.v1.EventValidatorVote"
                | "penumbra.core.component.governance.v1.EventProposalWithdraw"
                | "penumbra.core.component.governance.v1.EventProposalPassed"
                | "penumbra.core.component.governance.v1.EventProposalFailed"
                | "penumbra.core.component.governance.v1.EventProposalSlashed"
                | "penumbra.core.component.governance.v1.EventProposalDepositClaim"
                | "penumbra.core.component.sct.v1.EventBlockRoot"
        )
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        match event.event.kind.as_str() {
            "penumbra.core.component.governance.v1.EventProposalSubmit" => {
                let pe = pb::EventProposalSubmit::from_event(event.as_ref())?;
                let proposal = pe
                    .submit
                    .ok_or_else(|| anyhow!("missing submit in event"))?
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_submit(dbtx, proposal, event.block_height).await?;
            }
            "penumbra.core.component.governance.v1.EventDelegatorVote" => {
                let pe = pb::EventDelegatorVote::from_event(event.as_ref())?;
                let vote = pe
                    .vote
                    .ok_or_else(|| anyhow!("missing vote in event"))?
                    .try_into()
                    .context("error converting delegator vote")?;
                let validator_identity_key = pe
                    .validator_identity_key
                    .ok_or_else(|| anyhow!("missing validator identity key in event"))?
                    .try_into()
                    .context("error converting validator identity key")?;
                handle_delegator_vote(dbtx, vote, validator_identity_key, event.block_height)
                    .await?;
            }
            "penumbra.core.component.governance.v1.EventValidatorVote" => {
                let pe = pb::EventValidatorVote::from_event(event.as_ref())?;
                let vote = pe
                    .vote
                    .ok_or_else(|| anyhow!("missing vote in event"))?
                    .try_into()
                    .context("error converting vote")?;
                handle_validator_vote(dbtx, vote, event.block_height).await?;
            }
            "penumbra.core.component.governance.v1.EventProposalWithdraw" => {
                let pe = pb::EventProposalWithdraw::from_event(event.as_ref())?;
                let proposal_withdraw: ProposalWithdraw = pe
                    .withdraw
                    .ok_or_else(|| anyhow!("missing withdraw in event"))?
                    .try_into()
                    .context("error converting proposal withdraw")?;
                let proposal = proposal_withdraw.proposal;
                let reason = proposal_withdraw.reason;
                handle_proposal_withdraw(dbtx, proposal, reason).await?;
            }
            "penumbra.core.component.governance.v1.EventProposalPassed" => {
                let pe = pb::EventProposalPassed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_passed(dbtx, proposal).await?;
            }
            "penumbra.core.component.governance.v1.EventProposalFailed" => {
                let pe = pb::EventProposalFailed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_failed(dbtx, proposal).await?;
            }
            "penumbra.core.component.governance.v1.EventProposalSlashed" => {
                let pe = pb::EventProposalSlashed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_slashed(dbtx, proposal).await?;
            }
            "penumbra.core.component.governance.v1.EventProposalDepositClaim" => {
                let pe = pb::EventProposalDepositClaim::from_event(event.as_ref())?;
                let deposit_claim = pe
                    .deposit_claim
                    .ok_or_else(|| anyhow!("missing deposit claim in event"))?
                    .try_into()
                    .context("error converting deposit claim")?;
                handle_proposal_deposit_claim(dbtx, deposit_claim).await?;
            }
            "penumbra.core.component.sct.v1.EventBlockRoot" => {
                let pe = sct_pb::EventBlockRoot::from_event(event.as_ref())?;
                handle_block_root(dbtx, pe.height).await?;
            }
            _ => {}
        }

        Ok(())
    }
}

async fn handle_proposal_submit(
    dbtx: &mut PgTransaction<'_>,
    proposal: Proposal,
    block_height: u64,
) -> Result<()> {
    use penumbra_governance::ProposalKind::*;
    let payload_type = match proposal.kind() {
        Signaling => "SIGNALING",
        Emergency => "EMERGENCY",
        ParameterChange => "PARAMETER_CHANGE",
        CommunityPoolSpend => "COMMUNITY_POOL_SPEND",
        UpgradePlan => "UPGRADE_PLAN",
        FreezeIbcClient => "FREEZE_IBC_CLIENT",
        UnfreezeIbcClient => "UNFREEZE_IBC_CLIENT",
    };
    let proposal_data =
        serde_json::to_value(ProposalPayloadToml::from(proposal.payload)).expect("can serialize");
    sqlx::query(
        "INSERT INTO governance_proposals (
            proposal_id, title, description, payload_type, payload_data,
            start_block_height, stage, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(proposal.id as i32)
    .bind(&proposal.title)
    .bind(&proposal.description)
    .bind(payload_type)
    .bind(proposal_data)
    .bind(block_height as i64)
    .bind("VOTING")
    .bind("VOTING")
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_delegator_vote(
    dbtx: &mut PgTransaction<'_>,
    vote: DelegatorVote,
    validator_identity_key: IdentityKey,
    block_height: u64,
) -> Result<()> {
    let vote_body = vote.body;
    let vote = match vote_body.vote {
        Vote::Yes => "YES",
        Vote::No => "NO",
        Vote::Abstain => "ABSTAIN",
    };

    sqlx::query(
        "INSERT INTO delegator_votes (
            proposal_id, validator_identity_key, vote, voting_power, block_height
        )
        VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(vote_body.proposal as i32)
    .bind(validator_identity_key.to_string())
    .bind(vote)
    .bind(vote_body.unbonded_amount.value() as i64)
    .bind(block_height as i64)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_validator_vote(
    dbtx: &mut PgTransaction<'_>,
    vote: ValidatorVote,
    block_height: u64,
) -> Result<()> {
    let vote_body = vote.body;
    let vote = match vote_body.vote {
        Vote::Yes => "YES",
        Vote::No => "NO",
        Vote::Abstain => "ABSTAIN",
    };

    sqlx::query(
        "INSERT INTO validator_votes (
            proposal_id, identity_key, vote, voting_power, block_height
        )
        VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(vote_body.proposal as i32)
    .bind(&vote_body.identity_key.to_string())
    .bind(vote)
    .bind(0i64) // Voting power for validators is not included in the vote event
    .bind(block_height as i64)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_withdraw(
    dbtx: &mut PgTransaction<'_>,
    proposal_id: u64,
    reason: String,
) -> Result<()> {
    sqlx::query(
        "UPDATE governance_proposals
        SET
            status = $1,
            is_withdrawn = $2,
            withdrawal_reason = $3
        WHERE proposal_id = $4",
    )
    .bind("WITHDRAWN")
    .bind(true)
    .bind(reason)
    .bind(proposal_id as i32)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_passed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    sqlx::query(
        "UPDATE governance_proposals
        SET
            stage = $1,
            status = $2,
            outcome = $3
        WHERE proposal_id = $4",
    )
    .bind("FINISHED")
    .bind("FINISHED")
    .bind("PASSED")
    .bind(proposal.id as i32)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_failed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    sqlx::query(
        "UPDATE governance_proposals
        SET
            stage = $1,
            status = $2,
            outcome = $3
        WHERE proposal_id = $4",
    )
    .bind("FINISHED")
    .bind("FINISHED")
    .bind("FAILED")
    .bind(proposal.id as i32)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_slashed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    sqlx::query(
        "UPDATE governance_proposals
        SET
            stage = $1,
            status = $2,
            outcome = $3
        WHERE proposal_id = $4",
    )
    .bind("FINISHED")
    .bind("FINISHED")
    .bind("SLASHED")
    .bind(proposal.id as i32)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_deposit_claim(
    dbtx: &mut PgTransaction<'_>,
    deposit_claim: ProposalDepositClaim,
) -> Result<()> {
    sqlx::query(
        "UPDATE governance_proposals
        SET
            status = $1
        WHERE proposal_id = $2 AND status = $3",
    )
    .bind("CLAIMED")
    .bind(deposit_claim.proposal as i32)
    .bind("FINISHED")
    .execute(dbtx.as_mut())
    .await?;

    // Check if the update was successful
    if sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM governance_proposals WHERE proposal_id = $1 AND status = $2",
    )
    .bind(deposit_claim.proposal as i32)
    .bind("CLAIMED")
    .fetch_one(dbtx.as_mut())
    .await?
        == 0
    {
        anyhow::bail!("Failed to update proposal status to CLAIMED. The proposal might not exist or wasn't in the FINISHED state.");
    }

    Ok(())
}

async fn handle_block_root(dbtx: &mut PgTransaction<'_>, height: u64) -> Result<()> {
    sqlx::query(
        "INSERT INTO current_block_height (height)
         VALUES ($1)
         ON CONFLICT (height) DO UPDATE
         SET height = EXCLUDED.height
         WHERE EXCLUDED.height > current_block_height.height",
    )
    .bind(height as i64)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}
