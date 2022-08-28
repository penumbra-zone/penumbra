use std::{
    collections::BTreeMap,
    io::{stdout, Write},
};

use anyhow::Result;
use penumbra_component::{
    governance::{
        proposal::{self, ProposalList},
        state_key::*,
    },
    stake::validator,
};
use penumbra_crypto::IdentityKey;
use penumbra_transaction::action::{Proposal, ProposalPayload, Vote};
use serde::Serialize;
use serde_json::json;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum ProposalCmd {
    /// List all proposals by number.
    List {
        /// Whether to include proposals which have already finished voting.
        #[clap(short, long)]
        inactive: bool,
    },
    /// Fetch the details of a proposal, as submitted to the chain.
    Fetch {
        /// The number of the proposal to show.
        proposal_id: u64,
    },
    /// Display the current state of a proposal.
    State {
        /// The number of the proposal to show.
        proposal_id: u64,
    },
    /// Display the voting period of a proposal.
    Period {
        /// The number of the proposal to show.
        proposal_id: u64,
    },
    /// Display the latest epoch's tally of votes on the proposal, in units of voting power.
    Tally {
        /// The number of the proposal to show.
        proposal_id: u64,
    },
    /// List the votes of the validators who have voted on a proposal.
    ValidatorVotes {
        /// The number of the proposal to display validator votes for.
        proposal_id: u64,
    },
}

impl ProposalCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        use ProposalCmd::*;

        let mut client = app.specific_client().await?;

        match self {
            List { inactive } => {
                let list: Vec<u64> = if *inactive {
                    let latest: u64 = client.key_proto(latest_proposal_id()).await?;
                    (0..=latest).collect()
                } else {
                    let unfinished: ProposalList =
                        client.key_domain(unfinished_proposals()).await?;
                    unfinished.proposals.into_iter().collect()
                };
                json(&list)?;
            }
            Fetch { proposal_id } => {
                let description: String =
                    client.key_proto(proposal_description(*proposal_id)).await?;
                let payload: ProposalPayload =
                    client.key_domain(proposal_payload(*proposal_id)).await?;
                let proposal = Proposal {
                    description,
                    payload,
                };
                json(&proposal)?;
            }
            State { proposal_id } => {
                let state: proposal::State =
                    client.key_domain(proposal_state(*proposal_id)).await?;
                json(&state)?;
            }
            Period { proposal_id } => {
                let start: u64 = client
                    .key_proto(proposal_voting_start(*proposal_id))
                    .await?;
                let end: u64 = client.key_proto(proposal_voting_end(*proposal_id)).await?;
                let period = json!({
                    "voting_start_block": start,
                    "voting_end_block": end,
                });
                json(&period)?;
            }
            ValidatorVotes { proposal_id } => {
                let voting_validators: validator::List =
                    client.key_domain(voting_validators(*proposal_id)).await?;

                let mut votes: BTreeMap<IdentityKey, Vote> = BTreeMap::new();
                for identity_key in voting_validators.0.iter() {
                    let vote: Vote = client
                        .key_domain(validator_vote(*proposal_id, *identity_key))
                        .await?;
                    votes.insert(*identity_key, vote);
                }
                json(&votes)?;
            }
            Tally { proposal_id: _ } => {
                todo!("vote tallying not yet implemented");
            }
        }

        Ok(())
    }
}

fn json<T: Serialize>(value: &T) -> Result<()> {
    let mut writer = stdout().lock();
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    Ok(())
}
