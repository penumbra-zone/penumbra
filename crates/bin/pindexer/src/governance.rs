use anyhow::{anyhow, Context, Result};
use cometindex::{
    async_trait,
    index::{EventBatch, EventBatchContext},
    sqlx, AppView, ContextualizedEvent, PgTransaction,
};
use penumbra_sdk_governance::{
    proposal::ProposalPayloadToml, proposal_state, DelegatorVote, Proposal, ProposalDepositClaim,
    ProposalWithdraw, ValidatorVote,
};
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    core::component::governance::v1::{self as pb},
    event::ProtoEvent,
};
use penumbra_sdk_stake::IdentityKey;

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

impl GovernanceProposals {
    async fn index_event(
        &self,
        dbtx: &mut PgTransaction<'_>,
        event: ContextualizedEvent<'_>,
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
            _ => {}
        }

        Ok(())
    }
}

#[async_trait]
impl AppView for GovernanceProposals {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        // Define table structures
        let tables = vec![
            (
                "governance_proposals",
                r"
                id SERIAL PRIMARY KEY,
                proposal_id INTEGER NOT NULL UNIQUE,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                kind JSONB NOT NULL,
                payload JSONB,
                start_block_height BIGINT NOT NULL,
                end_block_height BIGINT NOT NULL,
                state JSONB NOT NULL,
                proposal_deposit_amount BIGINT NOT NULL,
                withdrawn BOOLEAN DEFAULT FALSE,
                withdrawal_reason TEXT
                ",
            ),
            (
                "governance_validator_votes",
                r"
                id SERIAL PRIMARY KEY,
                proposal_id INTEGER NOT NULL,
                identity_key TEXT NOT NULL,
                vote JSONB NOT NULL,
                voting_power BIGINT NOT NULL,
                block_height BIGINT NOT NULL,
                FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
                ",
            ),
            (
                "governance_delegator_votes",
                r"
                id SERIAL PRIMARY KEY,
                proposal_id INTEGER NOT NULL,
                identity_key TEXT NOT NULL,
                vote JSONB NOT NULL,
                voting_power BIGINT NOT NULL,
                block_height BIGINT NOT NULL,
                FOREIGN KEY (proposal_id) REFERENCES governance_proposals(proposal_id)
                ",
            ),
        ];

        // Define indexes
        let indexes = vec![
            (
                "governance_proposals",
                "proposal_id",
                "idx_governance_proposals_id",
            ),
            (
                "governance_proposals",
                "title text_pattern_ops",
                "idx_governance_proposals_title",
            ),
            (
                "governance_proposals",
                "kind",
                "idx_governance_proposals_kind",
            ),
            (
                "governance_proposals",
                "start_block_height DESC",
                "idx_governance_proposals_start_block_height",
            ),
            (
                "governance_proposals",
                "end_block_height DESC",
                "idx_governance_proposals_end_block_height",
            ),
            (
                "governance_proposals",
                "state",
                "idx_governance_proposals_state",
            ),
            (
                "governance_proposals",
                "withdrawn",
                "idx_governance_proposals_withdrawn",
            ),
            (
                "governance_validator_votes",
                "proposal_id",
                "idx_governance_validator_votes_proposal_id",
            ),
            (
                "governance_validator_votes",
                "identity_key",
                "idx_governance_validator_votes_identity_key",
            ),
            (
                "governance_validator_votes",
                "vote",
                "idx_governance_validator_votes_vote",
            ),
            (
                "governance_validator_votes",
                "voting_power",
                "idx_governance_validator_votes_voting_power",
            ),
            (
                "governance_validator_votes",
                "block_height",
                "idx_governance_validator_votes_block_height",
            ),
            (
                "governance_delegator_votes",
                "proposal_id",
                "idx_governance_delegator_votes_proposal_id",
            ),
            (
                "governance_delegator_votes",
                "identity_key",
                "idx_governance_delegator_votes_identity_key",
            ),
            (
                "governance_delegator_votes",
                "vote",
                "idx_governance_delegator_votes_vote",
            ),
            (
                "governance_delegator_votes",
                "voting_power",
                "idx_governance_delegator_votes_voting_power",
            ),
            (
                "governance_delegator_votes",
                "block_height",
                "idx_governance_delegator_votes_block_height",
            ),
        ];

        async fn create_table(
            dbtx: &mut PgTransaction<'_>,
            table_name: &str,
            structure: &str,
        ) -> Result<()> {
            let query = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, structure);
            sqlx::query(&query).execute(dbtx.as_mut()).await?;
            Ok(())
        }

        async fn create_index(
            dbtx: &mut PgTransaction<'_>,
            table_name: &str,
            column: &str,
            index_name: &str,
        ) -> Result<()> {
            let query = format!(
                "CREATE INDEX IF NOT EXISTS {} ON {}({})",
                index_name, table_name, column
            );
            sqlx::query(&query).execute(dbtx.as_mut()).await?;
            Ok(())
        }

        // Create tables
        for (table_name, table_structure) in tables {
            create_table(dbtx, table_name, table_structure).await?;
        }

        // Create indexes
        for (table_name, column, index_name) in indexes {
            create_index(dbtx, table_name, column, index_name).await?;
        }

        Ok(())
    }

    fn name(&self) -> String {
        "governance".to_string()
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
            proposal_id, title, description, kind, payload, start_block_height, end_block_height, state, proposal_deposit_amount
        )
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
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
    .bind(proposal.id as i64)
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
    .bind(proposal.id as i64)
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
    let current_state: serde_json::Value = sqlx::query_scalar(
        "SELECT state
        FROM governance_proposals
        WHERE proposal_id = $1",
    )
    .bind(deposit_claim.proposal as i64)
    .fetch_one(dbtx.as_mut())
    .await?;

    let current_state: proposal_state::State = serde_json::from_value(current_state)?;

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
