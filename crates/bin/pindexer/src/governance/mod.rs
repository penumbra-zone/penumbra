use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use cometindex::{async_trait, sqlx, AppView, ContextualizedEvent, PgTransaction};
use penumbra_governance::{
    proposal::ProposalPayloadToml, proposal_state, DelegatorVote, Proposal, ProposalDepositClaim,
    ProposalWithdraw, ValidatorVote,
};
use penumbra_num::Amount;
use penumbra_proto::{
    core::component::governance::v1::{self as pb},
    event::ProtoEvent,
};
use penumbra_stake::IdentityKey;
use sqlx::{PgPool, Postgres, Transaction};

/// One of the possible events that we care about.
#[derive(Clone, Debug)]
enum Event {
    ProposalSubmit {
        proposal: Proposal,
        deposit_amount: Amount,
        start_block_height: u64,
        end_block_height: u64,
    },
    DelegatorVote {
        vote: DelegatorVote,
        identity_key: IdentityKey,
        block_height: u64,
    },
    ValidatorVote {
        vote: ValidatorVote,
        voting_power: u64,
        block_height: u64,
    },
    ProposalWithdraw {
        proposal_id: u64,
        reason: String,
    },
    ProposalPassed {
        proposal: Proposal,
    },
    ProposalFailed {
        proposal: Proposal,
    },
    ProposalSlashed {
        proposal: Proposal,
    },
    ProposalDepositClaim {
        deposit_claim: ProposalDepositClaim,
    },
}

impl Event {
    const NAMES: [&'static str; 8] = [
        "penumbra.core.component.governance.v1.EventProposalSubmit",
        "penumbra.core.component.governance.v1.EventDelegatorVote",
        "penumbra.core.component.governance.v1.EventValidatorVote",
        "penumbra.core.component.governance.v1.EventProposalWithdraw",
        "penumbra.core.component.governance.v1.EventProposalPassed",
        "penumbra.core.component.governance.v1.EventProposalFailed",
        "penumbra.core.component.governance.v1.EventProposalSlashed",
        "penumbra.core.component.governance.v1.EventProposalDepositClaim",
    ];

    async fn index<'d>(&self, dbtx: &mut Transaction<'d, Postgres>) -> anyhow::Result<()> {
        // suboptimal, but makes the rest of this ported code work more or less
        match self.clone() {
            Event::ProposalSubmit {
                proposal,
                deposit_amount,
                start_block_height,
                end_block_height,
            } => {
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
            }
            Event::DelegatorVote {
                vote,
                identity_key,
                block_height,
            } => {
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
            }
            Event::ValidatorVote {
                vote,
                voting_power,
                block_height,
            } => {
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
            }
            Event::ProposalWithdraw {
                proposal_id,
                reason,
            } => {
                sqlx::query(
                    "UPDATE governance_proposals
                 SET withdrawn = TRUE, withdrawal_reason = $2
                 WHERE proposal_id = $1",
                )
                .bind(proposal_id as i64)
                .bind(&reason)
                .execute(dbtx.as_mut())
                .await?;
            }
            Event::ProposalPassed { proposal } => {
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
            }
            Event::ProposalFailed { proposal } => {
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
            }
            Event::ProposalSlashed { proposal } => {
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
            }
            Event::ProposalDepositClaim { deposit_claim } => {
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
            }
        };
        Ok(())
    }
}

impl<'a> TryFrom<&'a ContextualizedEvent> for Event {
    type Error = anyhow::Error;

    fn try_from(event: &'a ContextualizedEvent) -> Result<Self, Self::Error> {
        let block_height = event.block_height;
        match event.event.kind.as_str() {
            // Proposal Submit
            x if x == Event::NAMES[0] => {
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
                Ok(Self::ProposalSubmit {
                    proposal,
                    deposit_amount,
                    start_block_height,
                    end_block_height,
                })
            }
            // Delegator Vote
            x if x == Event::NAMES[1] => {
                let pe = pb::EventDelegatorVote::from_event(event.as_ref())?;
                let vote = pe
                    .vote
                    .ok_or_else(|| anyhow!("missing vote in event"))?
                    .try_into()
                    .context("error converting delegator vote")?;
                let identity_key = pe
                    .validator_identity_key
                    .ok_or_else(|| anyhow!("missing validator identity key in event"))?
                    .try_into()
                    .context("error converting validator identity key")?;
                Ok(Self::DelegatorVote {
                    vote,
                    identity_key,
                    block_height,
                })
            }
            // Validator vote
            x if x == Event::NAMES[2] => {
                let pe = pb::EventValidatorVote::from_event(event.as_ref())?;
                let voting_power = pe.voting_power;
                let vote = pe
                    .vote
                    .ok_or_else(|| anyhow!("missing vote in event"))?
                    .try_into()
                    .context("error converting vote")?;
                Ok(Self::ValidatorVote {
                    vote,
                    voting_power,
                    block_height,
                })
            }
            // Propopsal Withraw
            x if x == Event::NAMES[3] => {
                let pe = pb::EventProposalWithdraw::from_event(event.as_ref())?;
                let proposal_withdraw: ProposalWithdraw = pe
                    .withdraw
                    .ok_or_else(|| anyhow!("missing withdraw in event"))?
                    .try_into()
                    .context("error converting proposal withdraw")?;
                let proposal_id = proposal_withdraw.proposal;
                let reason = proposal_withdraw.reason;
                Ok(Self::ProposalWithdraw {
                    proposal_id,
                    reason,
                })
            }
            // Proposal Passed
            x if x == Event::NAMES[4] => {
                let pe = pb::EventProposalPassed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                Ok(Self::ProposalPassed { proposal })
            }
            // Proposal Failed
            x if x == Event::NAMES[5] => {
                let pe = pb::EventProposalFailed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                Ok(Self::ProposalFailed { proposal })
            }
            // Proposal Slashed
            x if x == Event::NAMES[6] => {
                let pe = pb::EventProposalSlashed::from_event(event.as_ref())?;
                let proposal = pe
                    .proposal
                    .ok_or_else(|| anyhow!("missing proposal in event"))?
                    .try_into()
                    .context("error converting proposal")?;
                Ok(Self::ProposalSlashed { proposal })
            }
            // Proposal Deposit Claim
            x if x == Event::NAMES[7] => {
                let pe = pb::EventProposalDepositClaim::from_event(event.as_ref())?;
                let deposit_claim = pe
                    .deposit_claim
                    .ok_or_else(|| anyhow!("missing deposit claim in event"))?
                    .try_into()
                    .context("error converting deposit claim")?;
                Ok(Self::ProposalDepositClaim { deposit_claim })
            }
            x => Err(anyhow!(format!("unrecognized event kind: {x}"))),
        }
    }
}

#[derive(Debug)]
pub struct Component {
    event_strings: HashSet<&'static str>,
}

impl Component {
    pub fn new() -> Self {
        let event_strings = Event::NAMES.into_iter().collect();
        Self { event_strings }
    }
}

#[async_trait]
impl AppView for Component {
    async fn init_chain(
        &self,
        dbtx: &mut PgTransaction,
        _app_state: &serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        for statement in include_str!("governance.sql").split(";") {
            sqlx::query(statement).execute(dbtx.as_mut()).await?;
        }
        Ok(())
    }

    fn is_relevant(&self, type_str: &str) -> bool {
        self.event_strings.contains(&type_str)
    }

    async fn index_event(
        &self,
        dbtx: &mut PgTransaction,
        event: &ContextualizedEvent,
        _src_db: &PgPool,
    ) -> Result<(), anyhow::Error> {
        let event = Event::try_from(event)?;
        event.index(dbtx).await
    }
}
