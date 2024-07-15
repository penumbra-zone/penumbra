use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_governance::{
    proposal::ProposalPayloadToml, proposal_state, DelegatorVote, Proposal, ProposalDepositClaim,
    ProposalWithdraw, ValidatorVote,
};
use penumbra_num::Amount;
use penumbra_proto::{
    core::component::{
        governance::v1::{self as pb},
        sct::v1 as sct_pb,
    },
    event::ProtoEvent,
};
use penumbra_stake::IdentityKey;

#[derive(Debug)]
pub struct GovernanceProposals {}

const EVENT_PROPOSAL_SUBMIT: &str = "penumbra.core.component.governance.v1.EventProposalSubmit";
const EVENT_DELEGATOR_VOTE: &str = "penumbra.core.component.governance.v1.EventDelegatorVote";
const EVENT_VALIDATOR_VOTE: &str = "penumbra.core.component.governance.v1.EventValidatorVote";
const EVENT_PROPOSAL_WITHDRAW: &str = "penumbra.core.component.governance.v1.EventProposalWithdraw";
const EVENT_PROPOSAL_PASSED: &str = "penumbra.core.component.governance.v1.EventProposalPassed";
const EVENT_PROPOSAL_FAILED: &str = "penumbra.core.component.governance.v1.EventProposalFailed";
const EVENT_PROPOSAL_SLASHED: &str = "penumbra.core.component.governance.v1.EventProposalSlashed";
const EVENT_PROPOSAL_DEPOSIT_CLAIM: &str =
    "penumbra.core.component.governance.v1.EventProposalDepositClaim";
const EVENT_BLOCK_ROOT: &str = "penumbra.core.component.sct.v1.EventBlockRoot";
const ALL_RELEVANT_EVENTS: &[&str] = &[
    EVENT_PROPOSAL_SUBMIT,
    EVENT_DELEGATOR_VOTE,
    EVENT_VALIDATOR_VOTE,
    EVENT_PROPOSAL_WITHDRAW,
    EVENT_PROPOSAL_PASSED,
    EVENT_PROPOSAL_FAILED,
    EVENT_PROPOSAL_SLASHED,
    EVENT_PROPOSAL_DEPOSIT_CLAIM,
    EVENT_BLOCK_ROOT,
];

#[async_trait]
impl AppView for GovernanceProposals {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(r"
CREATE TABLE governance_proposals (
    id SERIAL PRIMARY KEY,
    -- The on-chain proposal ID
    proposal_id INTEGER NOT NULL,
    -- The proposal title
    title TEXT NOT NULL,
    -- The proposal description
    description TEXT,
    -- The kind of the proposal
    kind TEXT NOT NULL,
    -- The proposal payload
    payload JSONB,
    -- The height at which voting starts
    start_block_height BIGINT,
    -- The height at which voting ends
    end_block_height BIGINT,
    -- The position of the Tiered Commitment Tree at the start of the proposal
    start_position BIGINT,
    -- The state of the proposal
    state TEXT NOT NULL,
    -- The amount of the deposit which will be slashed if the proposal is rejected
    proposal_deposit_amount BIGINT,
    -- Whether the proposal has been withdrawn
    withdrawn BOOLEAN DEFAULT FALSE,
    -- The reason for the withdrawal (null if the proposal has not been withdrawn)
    withdrawal_reason TEXT
);

CREATE INDEX idx_governance_proposals_id ON governance_proposals(proposal_id);
CREATE INDEX idx_governance_proposals_title ON governance_proposals(title text_pattern_ops);
CREATE INDEX idx_governance_proposals_kind ON governance_proposals(kind);
CREATE INDEX idx_governance_proposals_start_block_height ON governance_proposals(start_block_height DESC);
CREATE INDEX idx_governance_proposals_end_block_height ON governance_proposals(end_block_height DESC);
CREATE INDEX idx_governance_proposals_status ON governance_proposals(status);
CREATE INDEX idx_governance_proposals_outcome ON governance_proposals(outcome);
CREATE INDEX idx_governance_proposals_withdrawn ON governance_proposals(withdrawn);

CREATE TABLE governance_validator_votes (
    id SERIAL PRIMARY KEY,
    -- The on-chain proposal ID
    proposal_id INTEGER NOT NULL,
    -- The identity key of the validator
    identity_key TEXT NOT NULL,
    -- The vote of the validator
    vote TEXT NOT NULL,
    -- The voting power of the validator
    voting_power BIGINT NOT NULL,
    -- The height at which the vote was cast
    block_height BIGINT NOT NULL,
    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX idx_governance_validator_votes_proposal_id ON governance_validator_votes(proposal_id);
CREATE INDEX idx_governance_validator_votes_identity_key ON governance_validator_votes(identity_key);
CREATE INDEX idx_governance_validator_votes_vote ON governance_validator_votes(vote);
CREATE INDEX idx_governance_validator_votes_voting_power ON governance_validator_votes(voting_power);
CREATE INDEX idx_governance_validator_votes_block_height ON governance_validator_votes(block_height);

CREATE TABLE governance_delegator_votes (
    id SERIAL PRIMARY KEY,
    -- The on-chain proposal ID
    proposal_id INTEGER NOT NULL,
    -- The identity key of the validator to which the delegator is delegating
    identity_key TEXT NOT NULL,
    -- The vote of the delegator
    vote TEXT NOT NULL,
    -- The voting power of the delegator
    voting_power BIGINT NOT NULL,
    -- The height at which the vote was cast
    block_height BIGINT NOT NULL,
    FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
);

CREATE INDEX idx_governance_delegator_votes_proposal_id ON governance_delegator_votes(proposal_id);
CREATE INDEX idx_governance_delegator_votes_identity_key ON governance_delegator_votes(identity_key);
CREATE INDEX idx_governance_delegator_votes_vote ON governance_delegator_votes(vote);
CREATE INDEX idx_governance_delegator_votes_voting_power ON governance_delegator_votes(voting_power);
CREATE INDEX idx_governance_delegator_votes_block_height ON governance_delegator_votes(block_height);
            ")
            .execute(dbtx.as_mut())
            .await?;
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        ALL_RELEVANT_EVENTS.contains(&type_str)
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
    ) -> Result<(), anyhow::Error> {
        match event.event.kind.as_str() {
            EVENT_PROPOSAL_SUBMIT => {
                let pe = pb::EventProposalSubmit::from_event(event.as_ref())?;
                let start_block_height = pe.start_height;
                let end_block_height = pe.end_height;
                let submit = pe
                    .submit
                    .ok_or_else(|| anyhow!("missing submit in event"))?;
                let deposit_amount = submit
                    .deposit_amount
                    .ok_or_else(|| anyhow!("missing deposit amount in event"))?
                    .try_into()
                    .context("error converting deposit amount")?;
                let proposal = submit
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_submit(
                    dbtx,
                    proposal,
                    deposit_amount,
                    start_block_height,
                    end_block_height,
                    event.block_height,
                )
                .await?;
            }
            EVENT_DELEGATOR_VOTE => {
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
            EVENT_VALIDATOR_VOTE => {
                let pe = pb::EventValidatorVote::from_event(event.as_ref())?;
                let voting_power = pe.voting_power;
                let vote = pe
                    .vote
                    .ok_or_else(|| anyhow!("missing vote in event"))?
                    .try_into()
                    .context("error converting vote")?;
                handle_validator_vote(dbtx, vote, voting_power, event.block_height).await?;
            }
            EVENT_PROPOSAL_WITHDRAW => {
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
            EVENT_PROPOSAL_PASSED => {
                let pe = pb::EventProposalPassed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_passed(dbtx, proposal).await?;
            }
            EVENT_PROPOSAL_FAILED => {
                let pe = pb::EventProposalFailed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_failed(dbtx, proposal).await?;
            }
            EVENT_PROPOSAL_SLASHED => {
                let pe = pb::EventProposalSlashed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                handle_proposal_slashed(dbtx, proposal).await?;
            }
            EVENT_PROPOSAL_DEPOSIT_CLAIM => {
                let pe = pb::EventProposalDepositClaim::from_event(event.as_ref())?;
                let deposit_claim = pe
                    .deposit_claim
                    .ok_or_else(|| anyhow!("missing deposit claim in event"))?
                    .try_into()
                    .context("error converting deposit claim")?;
                handle_proposal_deposit_claim(dbtx, deposit_claim).await?;
            }
            EVENT_BLOCK_ROOT => {
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
    deposit_amount: Amount,
    start_block_height: u64,
    end_block_height: u64,
    _block_height: u64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO governance_proposals (
            proposal_id, title, description, kind, payload, start_block_height, end_block_height, start_position, state, proposal_deposit_amount
        )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (proposal_id) DO NOTHING",
    )
    .bind(proposal.id as i64)
    .bind(&proposal.title)
    .bind(&proposal.description)
    .bind(serde_json::to_value(proposal.kind())?)
    .bind(serde_json::to_value(ProposalPayloadToml::from(proposal.payload))?)
    .bind(start_block_height as i64)
    .bind(end_block_height as i64)
    .bind(serde_json::to_value(proposal_state::State::Voting)?)
    .bind(deposit_amount.value() as i64)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_delegator_vote(
    dbtx: &mut PgTransaction<'_>,
    vote: DelegatorVote,
    identity_key: IdentityKey,
    block_height: u64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO governance_delegator_votes (
            proposal_id, identity_key, vote, voting_power, block_height
        )
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(vote.body.proposal as i64)
    .bind(&identity_key.to_string())
    .bind(serde_json::to_value(vote.body.vote)?)
    .bind(vote.body.unbonded_amount.value() as i64)
    .bind(block_height as i64)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_validator_vote(
    dbtx: &mut PgTransaction<'_>,
    vote: ValidatorVote,
    voting_power: u64,
    block_height: u64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO governance_validator_votes (
            proposal_id, identity_key, vote, voting_power, block_height
        )
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(vote.body.proposal as i64)
    .bind(&vote.body.identity_key.to_string())
    .bind(serde_json::to_value(vote.body.vote)?)
    .bind(voting_power as i64)
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
         SET withdrawn = TRUE, withdrawal_reason = $2
         WHERE proposal_id = $1",
    )
    .bind(proposal_id as i64)
    .bind(&reason)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_passed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    sqlx::query(
        "UPDATE governance_proposals
         SET state = $2
         WHERE proposal_id = $1",
    )
    .bind(proposal.id as i64)
    .bind(serde_json::to_value(proposal_state::State::Finished {
        outcome: proposal_state::Outcome::Passed,
    })?)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_failed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    // Determine if the proposal was withdrawn before it concluded, and if so, why
    let reason: Option<String> = sqlx::query_scalar(
        "SELECT withdrawal_reason
         FROM governance_proposals
         WHERE proposal_id = $1 AND withdrawn = TRUE
         LIMIT 1",
    )
    .fetch_optional(dbtx.as_mut())
    .await?;
    let withdrawn = proposal_state::Withdrawn::from(reason);

    sqlx::query(
        "UPDATE governance_proposals
         SET state = $2
         WHERE proposal_id = $1",
    )
    .bind(proposal.id as i64)
    .bind(serde_json::to_value(proposal_state::State::Finished {
        outcome: proposal_state::Outcome::Failed { withdrawn },
    })?)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_slashed(dbtx: &mut PgTransaction<'_>, proposal: Proposal) -> Result<()> {
    // Determine if the proposal was withdrawn before it concluded, and if so, why
    let reason: Option<String> = sqlx::query_scalar(
        "SELECT withdrawal_reason
         FROM governance_proposals
         WHERE proposal_id = $1 AND withdrawn = TRUE
         LIMIT 1",
    )
    .fetch_optional(dbtx.as_mut())
    .await?;
    let withdrawn = proposal_state::Withdrawn::from(reason);

    sqlx::query(
        "UPDATE governance_proposals
         SET state = $2
         WHERE proposal_id = $1",
    )
    .bind(proposal.id as i64)
    .bind(serde_json::to_value(proposal_state::State::Finished {
        outcome: proposal_state::Outcome::Slashed { withdrawn },
    })?)
    .execute(dbtx.as_mut())
    .await?;

    Ok(())
}

async fn handle_proposal_deposit_claim(
    dbtx: &mut PgTransaction<'_>,
    deposit_claim: ProposalDepositClaim,
) -> Result<()> {
    let current_state: String = sqlx::query_scalar(
        "SELECT state
            FROM governance_proposals
            WHERE proposal_id = $1",
    )
    .bind(deposit_claim.proposal as i64)
    .fetch_one(dbtx.as_mut())
    .await?;
    let current_state: proposal_state::State = serde_json::from_str(&current_state)?;

    let outcome = match current_state {
        proposal_state::State::Finished { outcome } => outcome,
        _ => {
            return Err(anyhow!(
                "proposal {} is not in a finished state",
                deposit_claim.proposal
            ))
        }
    };

    sqlx::query(
        "UPDATE governance_proposals
         SET state = $2
         WHERE proposal_id = $1",
    )
    .bind(deposit_claim.proposal as i64)
    .bind(serde_json::to_value(proposal_state::State::Claimed {
        outcome,
    })?)
    .execute(dbtx.as_mut())
    .await?;

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
