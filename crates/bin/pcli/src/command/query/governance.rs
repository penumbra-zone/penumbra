use std::{
    collections::BTreeMap,
    io::{stdout, Write},
};

use anyhow::{Context, Result};
use futures::TryStreamExt;
use penumbra_sdk_governance::Vote;
use penumbra_sdk_proto::core::component::governance::v1::{
    query_service_client::QueryServiceClient as GovernanceQueryServiceClient,
    AllTalliedDelegatorVotesForProposalRequest, ProposalDataRequest, ProposalListRequest,
    ProposalListResponse, ValidatorVotesRequest, ValidatorVotesResponse,
    VotingPowerAtProposalStartRequest,
};
use penumbra_sdk_stake::IdentityKey;
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
}

#[derive(Debug, clap::Subcommand)]
pub enum PerProposalCmd {
    /// Fetch the details of a proposal, as submitted to the chain.
    Definition,
    /// Display the current state of a proposal.
    State,
    /// Display the voting period of a proposal.
    Period,
    /// Display the most recent tally of votes on the proposal.
    Tally,
}

impl GovernanceCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        // use PerProposalCmd::*;

        let mut client = GovernanceQueryServiceClient::new(app.pd_channel().await?);
        match self {
            GovernanceCmd::ListProposals { inactive } => {
                let proposals: Vec<ProposalListResponse> = client
                    .proposal_list(ProposalListRequest {
                        inactive: *inactive,
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .try_collect::<Vec<_>>()
                    .await
                    .context("cannot process proposal list data")?;
                let mut writer = stdout();
                for proposal_response in proposals {
                    let proposal = proposal_response
                        .proposal
                        .expect("proposal should always be set");
                    let proposal_title = proposal.title;

                    let proposal_state = proposal_response
                        .state
                        .expect("proposal state should always be set");

                    let proposal_id = proposal.id;

                    writeln!(
                        writer,
                        "#{proposal_id} {proposal_state:?}    {proposal_title}"
                    )?;
                }
                Ok(())
            }
            GovernanceCmd::Proposal { proposal_id, query } => {
                match query {
                    &PerProposalCmd::Definition => {
                        let proposal = client
                            .proposal_data(ProposalDataRequest {
                                proposal_id: *proposal_id,
                                ..Default::default()
                            })
                            .await?
                            .into_inner();
                        toml(
                            &proposal
                                .proposal
                                .expect("proposal should always be populated"),
                        )?;
                    }
                    PerProposalCmd::State => {
                        let proposal = client
                            .proposal_data(ProposalDataRequest {
                                proposal_id: *proposal_id,
                                ..Default::default()
                            })
                            .await?
                            .into_inner();
                        json(
                            &proposal
                                .state
                                .expect("proposal state should always be populated"),
                        )?;
                    }
                    PerProposalCmd::Period => {
                        let proposal = client
                            .proposal_data(ProposalDataRequest {
                                proposal_id: *proposal_id,
                                ..Default::default()
                            })
                            .await?
                            .into_inner();
                        let start: u64 = proposal.start_block_height;
                        let end: u64 = proposal.end_block_height;
                        let period = json!({
                            "voting_start_block": start,
                            "voting_end_block": end,
                        });
                        json(&period)?;
                    }
                    PerProposalCmd::Tally => {
                        let validator_votes: Vec<ValidatorVotesResponse> = client
                            .validator_votes(ValidatorVotesRequest {
                                proposal_id: *proposal_id,
                                ..Default::default()
                            })
                            .await?
                            .into_inner()
                            .try_collect::<Vec<_>>()
                            .await?;

                        let mut validator_votes_and_power: BTreeMap<IdentityKey, (Vote, u64)> =
                            BTreeMap::new();
                        for vote_response in validator_votes {
                            let identity_key: IdentityKey = vote_response
                                .identity_key
                                .expect("identity key must be set for vote response")
                                .try_into()?;
                            let vote: Vote = vote_response
                                .vote
                                .expect("vote must be set for vote response")
                                .try_into()?;
                            let power: u64 = client
                                .voting_power_at_proposal_start(VotingPowerAtProposalStartRequest {
                                    proposal_id: *proposal_id,
                                    identity_key: Some(identity_key.into()),
                                    ..Default::default()
                                })
                                .await
                                .context("Error looking for validator power")?
                                .into_inner()
                                .voting_power;

                            validator_votes_and_power.insert(identity_key, (vote, power));
                        }

                        let mut delegator_tallies: BTreeMap<
                            IdentityKey,
                            penumbra_sdk_governance::Tally,
                        > = client
                            .all_tallied_delegator_votes_for_proposal(
                                AllTalliedDelegatorVotesForProposalRequest {
                                    proposal_id: *proposal_id,
                                    ..Default::default()
                                },
                            )
                            .await?
                            .into_inner()
                            .map_ok(|response| {
                                let identity_key: IdentityKey = response
                                    .identity_key
                                    .expect("identity key must be set for vote response")
                                    .try_into()?;
                                let tally: penumbra_sdk_governance::Tally = response
                                    .tally
                                    .expect("tally must be set for vote response")
                                    .try_into()?;
                                Ok::<(IdentityKey, penumbra_sdk_governance::Tally), anyhow::Error>(
                                    (identity_key, tally),
                                )
                            })
                            // TODO: double iterator here is suboptimal but trying to collect
                            // `Result<Vec<_>>` was annoying
                            .try_collect::<Vec<_>>()
                            .await?
                            .into_iter()
                            .collect::<Result<BTreeMap<_, _>>>()?;

                        // Combine the two mappings
                        let mut total = penumbra_sdk_governance::Tally::default();
                        let mut all_votes_and_power: BTreeMap<String, serde_json::Value> =
                            BTreeMap::new();
                        for (identity_key, (vote, power)) in validator_votes_and_power.into_iter() {
                            all_votes_and_power.insert(identity_key.to_string(), {
                                let mut map = serde_json::Map::new();
                                map.insert(
                                    "validator".to_string(),
                                    json!({
                                        vote.to_string(): power,
                                    }),
                                );
                                let delegator_tally =
                                    if let Some(tally) = delegator_tallies.remove(&identity_key) {
                                        map.insert("delegators".to_string(), json_tally(&tally));
                                        tally
                                    } else {
                                        Default::default()
                                    };
                                // Subtract delegator total from validator power, then add delegator
                                // tally in to get the total tally for this validator:
                                let sub_total = penumbra_sdk_governance::Tally::from((
                                    vote,
                                    power - delegator_tally.total(),
                                )) + delegator_tally;
                                map.insert("sub_total".to_string(), json_tally(&sub_total));
                                total += sub_total;
                                map.into()
                            });
                        }
                        for (identity_key, tally) in delegator_tallies.into_iter() {
                            all_votes_and_power.insert(identity_key.to_string(), {
                                let mut map = serde_json::Map::new();
                                let sub_total = tally;
                                map.insert("delegators".to_string(), json_tally(&tally));
                                map.insert("sub_total".to_string(), json_tally(&sub_total));
                                total += sub_total;
                                map.into()
                            });
                        }

                        json(&json!({
                        "total": json_tally(&total),
                        "details": all_votes_and_power,
                        }))?;
                    }
                };
                Ok(())
            }
        }
    }
}

fn json<T: Serialize>(value: &T) -> Result<()> {
    let mut writer = stdout();
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    Ok(())
}

fn json_tally(tally: &penumbra_sdk_governance::Tally) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if tally.yes() > 0 {
        map.insert("yes".to_string(), tally.yes().into());
    }
    if tally.no() > 0 {
        map.insert("no".to_string(), tally.no().into());
    }
    if tally.abstain() > 0 {
        map.insert("abstain".to_string(), tally.abstain().into());
    }
    map.into()
}

fn toml<T: Serialize>(value: &T) -> Result<()> {
    let mut writer = stdout();
    let string = toml::to_string_pretty(value)?;
    writer.write_all(string.as_bytes())?;
    Ok(())
}
