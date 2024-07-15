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
    todo!()
}

async fn handle_delegator_vote(
    dbtx: &mut PgTransaction<'_>,
    vote: DelegatorVote,
    validator_identity_key: IdentityKey,
    block_height: u64,
) -> Result<()> {
    todo!()
}

async fn handle_validator_vote(
    dbtx: &mut PgTransaction<'_>,
    vote: ValidatorVote,
    block_height: u64,
) -> Result<()> {
    todo!()
}

async fn handle_proposal_withdraw(
    dbtx: &mut PgTransaction<'_>,
    proposal_id: u64,
    reason: String,
) -> Result<()> {
    todo!()
}

async fn handle_proposal_passed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    todo!()
}

async fn handle_proposal_failed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    todo!()
}

async fn handle_proposal_slashed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    todo!()
}

async fn handle_proposal_deposit_claim(
    dbtx: &mut PgTransaction<'_>,
    deposit_claim: ProposalDepositClaim,
) -> Result<()> {
    todo!()
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
