use std::{
    collections::BTreeMap,
    io::{stdout, Write},
};

use anyhow::Result;
use futures::TryStreamExt;
use penumbra_component::{
    governance::{
        proposal::{self, ProposalList},
        state_key::*,
    },
    stake::validator,
};
use penumbra_crypto::IdentityKey;
use penumbra_proto::client::v1alpha1::MutableParametersRequest;
use penumbra_transaction::action::{Proposal, ProposalPayload, Vote};
use penumbra_view::ViewClient;
use serde::Serialize;
use serde_json::json;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum GovernanceCmd {
    /// List all governance proposals by number.
    ListProposals {
        /// Whether to include proposals which have already finished voting.
        #[clap(short, long)]
        inactive: bool,
    },
    /// Query for information about a particular proposal.
    Proposal {
        /// The proposal id to query.
        proposal_id: u64,
        /// The query to ask of it.
        #[clap(subcommand)]
        query: PerProposalCmd,
    },
    /// Query for the governance-modifiable chain parameters.
    Parameters,
}

#[derive(Debug, clap::Subcommand)]
pub enum PerProposalCmd {
    /// Fetch the details of a proposal, as submitted to the chain.
    Definition,
    /// Display the current state of a proposal.
    State,
    /// Display the voting period of a proposal.
    Period,
    /// Display the latest epoch's tally of votes on the proposal, in units of voting power.
    Tally,
    /// List the votes of the validators who have voted on a proposal.
    ValidatorVotes,
}

impl GovernanceCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        use PerProposalCmd::*;

        let mut client = app.specific_client().await?;

        match self {
            GovernanceCmd::Parameters => {
                let mut client = app.oblivious_client().await?;

                let params = client
                    .mutable_parameters(MutableParametersRequest {
                        chain_id: app.view().chain_params().await?.chain_id,
                    })
                    .await?
                    .into_inner()
                    .try_collect::<Vec<_>>()
                    .await?;

                println!("{}", serde_json::to_string_pretty(&params)?);
                json(&params)?;
            }
            GovernanceCmd::ListProposals { inactive } => {
                let proposal_id_list: Vec<u64> = if *inactive {
                    let latest: u64 = client.key_proto(latest_proposal_id()).await?;
                    (0..=latest).collect()
                } else {
                    let unfinished: ProposalList =
                        client.key_domain(unfinished_proposals()).await?;
                    unfinished.proposals.into_iter().collect()
                };

                let mut writer = stdout();
                for proposal_id in proposal_id_list {
                    let proposal_title: String =
                        client.key_proto(proposal_title(proposal_id)).await?;
                    let proposal_state: proposal::State =
                        client.key_domain(proposal_state(proposal_id)).await?;

                    writeln!(
                        writer,
                        "#{} {:?}    {}",
                        proposal_id, proposal_state, proposal_title
                    )?;
                }
            }
            GovernanceCmd::Proposal { proposal_id, query } => match query {
                Definition => {
                    let title: String = client.key_proto(proposal_title(*proposal_id)).await?;
                    let description: String =
                        client.key_proto(proposal_description(*proposal_id)).await?;
                    let payload: ProposalPayload =
                        client.key_domain(proposal_payload(*proposal_id)).await?;
                    let proposal = Proposal {
                        title,
                        description,
                        payload,
                    };
                    json(&proposal)?;
                }
                State => {
                    let state: proposal::State =
                        client.key_domain(proposal_state(*proposal_id)).await?;
                    json(&state)?;
                }
                Period => {
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
                ValidatorVotes => {
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
                Tally => {
                    todo!("vote tallying not yet implemented");
                }
            },
        }

        Ok(())
    }
}

fn json<T: Serialize>(value: &T) -> Result<()> {
    let mut writer = stdout();
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    Ok(())
}
