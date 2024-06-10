//! Contains functions related to the migration script of Testnet78.
use cnidarium::{Snapshot, StateDelta, Storage};
use futures::TryStreamExt as _;
use jmt::RootHash;
use pbjson_types::Any;
use penumbra_app::app::StateReadExt as _;
use penumbra_governance::StateReadExt as _;
use penumbra_proto::{DomainType as _, StateReadProto as _, StateWriteProto as _};
use penumbra_sct::component::clock::EpochRead as _;
use penumbra_stake::validator::Validator;
use std::path::PathBuf;
use tracing::instrument;

use crate::testnet::generate::TestnetConfig;

/// Run the full migration, given an export path and a start time for genesis.
///
/// Menu:
/// - Truncate various user-supplied `String` fields to a maximum length.
///   * Validators:
///    - `name` (140 bytes)
///    - `website` (70 bytes)
///    - `description` (280 bytes)
///   * Governance Parameter Changes:
///    - `key` (64 bytes)
///    - `value` (2048 bytes)
///    - `component` (64 bytes)
///   * Governance Proposals:
///    - `title` (80 bytes)
///    - `description` (10,000 bytes)
///   * Governance Proposal Withdrawals:
///    - `reason` (1024 bytes)
///   * Governance IBC Client Freeze Proposals:
///    - `client_id` (128 bytes)
///   * Governance IBC Client Unfreeze Proposals:
///    - `client_id` (128 bytes)
///   * Governance Signaling Proposals:
///    - `commit hash` (255 bytes)
#[instrument]
pub async fn migrate(
    storage: Storage,
    pd_home: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    // Setup:
    let initial_state = storage.latest_snapshot();
    let chain_id = initial_state.get_chain_id().await?;
    let root_hash = initial_state
        .root_hash()
        .await
        .expect("chain state has a root hash");
    let pre_upgrade_root_hash: RootHash = root_hash.into();
    let pre_upgrade_height = initial_state
        .get_block_height()
        .await
        .expect("chain state has a block height");
    let post_upgrade_height = pre_upgrade_height.wrapping_add(1);

    // We initialize a `StateDelta` and start by reaching into the JMT for all entries matching the
    // swap execution prefix. Then, we write each entry to the nv-storage.
    let mut delta = StateDelta::new(initial_state);
    let (migration_duration, post_upgrade_root_hash) = {
        let start_time = std::time::SystemTime::now();
        // Adjust the length of `Validator` fields.
        truncate_validator_fields(&mut delta).await?;

        // Adjust the length of governance proposal fields.
        truncate_proposal_fields(&mut delta).await?;

        // Adjust the length of governance proposal outcome fields.
        truncate_proposal_outcome_fields(&mut delta).await?;

        let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
        tracing::info!(?post_upgrade_root_hash, "post-migration root hash");

        (
            start_time.elapsed().expect("start time not set"),
            post_upgrade_root_hash,
        )
    };
    storage.release().await;

    // The migration is complete, now we need to generate a genesis file. To do this, we need
    // to lookup a validator view from the chain, and specify the post-upgrade app hash and
    // initial height.
    let app_state = penumbra_app::genesis::Content {
        chain_id,
        ..Default::default()
    };
    let mut genesis = TestnetConfig::make_genesis(app_state.clone()).expect("can make genesis");
    genesis.app_hash = post_upgrade_root_hash
        .0
        .to_vec()
        .try_into()
        .expect("infaillible conversion");
    genesis.initial_height = post_upgrade_height as i64;
    genesis.genesis_time = genesis_start.unwrap_or_else(|| {
        let now = tendermint::time::Time::now();
        tracing::info!(%now, "no genesis time provided, detecting a testing setup");
        now
    });
    let checkpoint = post_upgrade_root_hash.0.to_vec();
    let genesis = TestnetConfig::make_checkpoint(genesis, Some(checkpoint));

    let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
    tracing::info!("genesis: {}", genesis_json);
    let genesis_path = pd_home.join("genesis.json");
    std::fs::write(genesis_path, genesis_json).expect("can write genesis");

    let validator_state_path = pd_home.join("priv_validator_state.json");

    let fresh_validator_state = crate::testnet::generate::TestnetValidator::initial_state();
    std::fs::write(validator_state_path, fresh_validator_state).expect("can write validator state");

    tracing::info!(
        pre_upgrade_height,
        post_upgrade_height,
        ?pre_upgrade_root_hash,
        ?post_upgrade_root_hash,
        duration = migration_duration.as_secs(),
        "successful migration!"
    );

    Ok(())
}

///   * Validators:
///    - `name` (140 bytes)
///    - `website` (70 bytes)
///    - `description` (280 bytes)
async fn truncate_validator_fields(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let key_prefix_validators = penumbra_stake::state_key::validators::definitions::prefix();
    let all_validators = delta
        .prefix_proto::<Any>(&key_prefix_validators)
        .map_ok(|(k, v)| (k, Validator::decode(v.value).expect("only validators")))
        .try_collect::<Vec<(String, Validator)>>()
        .await?;

    for (key, mut validator) in all_validators {
        validator.name = truncate(&validator.name, 140).to_string();
        validator.website = truncate(&validator.website, 70).to_string();
        validator.description = truncate(&validator.description, 280).to_string();

        delta.put(key, validator);
    }

    Ok(())
}

///   * Governance Proposals:
///    - `title` (80 bytes)
///    - `description` (10,000 bytes)
///   * Governance Parameter Changes:
///    - `key` (64 bytes)
///    - `value` (2048 bytes)
///    - `component` (64 bytes)
///   * Governance IBC Client Freeze Proposals:
///    - `client_id` (128 bytes)
///   * Governance IBC Client Unfreeze Proposals:
///    - `client_id` (128 bytes)
///   * Governance Signaling Proposals:
///    - `commit hash` (255 bytes)
async fn truncate_proposal_fields(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let next_proposal_id: u64 = delta.next_proposal_id().await?;

    // Range each proposal and truncate the fields.
    for proposal_id in 0..next_proposal_id {
        let proposal = delta.proposal_definition(proposal_id).await?;

        if proposal.is_none() {
            break;
        }

        let mut proposal = proposal.unwrap();

        proposal.title = truncate(&proposal.title, 80).to_string();
        proposal.description = truncate(&proposal.description, 10_000).to_string();

        // Depending on the proposal type, we may need to truncate additional fields.
        match proposal.payload {
            penumbra_governance::ProposalPayload::Signaling { commit } => {
                proposal.payload = penumbra_governance::ProposalPayload::Signaling {
                    commit: commit.map(|commit| truncate(&commit, 255).to_string()),
                };
            }
            penumbra_governance::ProposalPayload::Emergency { halt_chain: _ } => {}
            penumbra_governance::ProposalPayload::ParameterChange(mut param_change) => {
                for (i, mut change) in param_change.changes.clone().into_iter().enumerate() {
                    let key = truncate(&change.key, 64).to_string();
                    let value = truncate(&change.value, 2048).to_string();
                    let component = truncate(&change.component, 64).to_string();

                    change.key = key;
                    change.value = value;
                    change.component = component;

                    param_change.changes[i] = change;
                }

                for (i, mut change) in param_change.preconditions.clone().into_iter().enumerate() {
                    let key = truncate(&change.key, 64).to_string();
                    let value = truncate(&change.value, 2048).to_string();
                    let component = truncate(&change.component, 64).to_string();

                    change.key = key;
                    change.value = value;
                    change.component = component;

                    param_change.preconditions[i] = change;
                }

                proposal.payload =
                    penumbra_governance::ProposalPayload::ParameterChange(param_change);
            }
            penumbra_governance::ProposalPayload::CommunityPoolSpend {
                transaction_plan: _,
            } => {}
            penumbra_governance::ProposalPayload::UpgradePlan { height: _ } => {}
            penumbra_governance::ProposalPayload::FreezeIbcClient { client_id } => {
                proposal.payload = penumbra_governance::ProposalPayload::FreezeIbcClient {
                    client_id: truncate(&client_id, 128).to_string(),
                };
            }
            penumbra_governance::ProposalPayload::UnfreezeIbcClient { client_id } => {
                proposal.payload = penumbra_governance::ProposalPayload::UnfreezeIbcClient {
                    client_id: truncate(&client_id, 128).to_string(),
                };
            }
        };

        // Store the truncated proposal data
        delta.put(
            penumbra_governance::state_key::proposal_definition(proposal_id),
            proposal.clone(),
        );
    }

    Ok(())
}

///   * Governance Proposal Withdrawals:
///    - `reason` (1024 bytes)
async fn truncate_proposal_outcome_fields(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let next_proposal_id: u64 = delta.next_proposal_id().await?;

    // Range each proposal outcome and truncate the fields.
    for proposal_id in 0..next_proposal_id {
        let proposal_state = delta.proposal_state(proposal_id).await?;

        if proposal_state.is_none() {
            break;
        }

        let mut proposal_state = proposal_state.unwrap();

        match proposal_state {
            penumbra_governance::proposal_state::State::Withdrawn { reason } => {
                proposal_state = penumbra_governance::proposal_state::State::Withdrawn {
                    reason: truncate(&reason, 1024).to_string(),
                };
            }
            penumbra_governance::proposal_state::State::Voting => {}
            penumbra_governance::proposal_state::State::Finished { ref outcome } => match outcome {
                penumbra_governance::proposal_state::Outcome::Passed => {}
                penumbra_governance::proposal_state::Outcome::Failed { withdrawn } => {
                    match withdrawn {
                        penumbra_governance::proposal_state::Withdrawn::No => {}
                        penumbra_governance::proposal_state::Withdrawn::WithReason { reason } => {
                            proposal_state = penumbra_governance::proposal_state::State::Finished {
                                outcome: penumbra_governance::proposal_state::Outcome::Failed {
                                    withdrawn:
                                        penumbra_governance::proposal_state::Withdrawn::WithReason {
                                            reason: truncate(&reason, 1024).to_string(),
                                        },
                                },
                            };
                        }
                    }
                }
                penumbra_governance::proposal_state::Outcome::Slashed { withdrawn } => {
                    match withdrawn {
                        penumbra_governance::proposal_state::Withdrawn::No => {}
                        penumbra_governance::proposal_state::Withdrawn::WithReason { reason } => {
                            proposal_state = penumbra_governance::proposal_state::State::Finished {
                                outcome: penumbra_governance::proposal_state::Outcome::Slashed {
                                    withdrawn:
                                        penumbra_governance::proposal_state::Withdrawn::WithReason {
                                            reason: truncate(&reason, 1024).to_string(),
                                        },
                                },
                            };
                        }
                    }
                }
            },
            penumbra_governance::proposal_state::State::Claimed { ref outcome } => match outcome {
                penumbra_governance::proposal_state::Outcome::Passed => {}
                penumbra_governance::proposal_state::Outcome::Failed { withdrawn } => {
                    match withdrawn {
                        penumbra_governance::proposal_state::Withdrawn::No => {}
                        penumbra_governance::proposal_state::Withdrawn::WithReason { reason } => {
                            proposal_state = penumbra_governance::proposal_state::State::Claimed {
                                outcome: penumbra_governance::proposal_state::Outcome::Failed {
                                    withdrawn:
                                        penumbra_governance::proposal_state::Withdrawn::WithReason {
                                            reason: truncate(&reason, 1024).to_string(),
                                        },
                                },
                            };
                        }
                    }
                }
                penumbra_governance::proposal_state::Outcome::Slashed { withdrawn } => {
                    match withdrawn {
                        penumbra_governance::proposal_state::Withdrawn::No => {}
                        penumbra_governance::proposal_state::Withdrawn::WithReason { reason } => {
                            proposal_state = penumbra_governance::proposal_state::State::Claimed {
                                outcome: penumbra_governance::proposal_state::Outcome::Slashed {
                                    withdrawn:
                                        penumbra_governance::proposal_state::Withdrawn::WithReason {
                                            reason: truncate(&reason, 1024).to_string(),
                                        },
                                },
                            };
                        }
                    }
                }
            },
        }

        // Store the truncated proposal state data
        delta.put(
            penumbra_governance::state_key::proposal_state(proposal_id),
            proposal_state.clone(),
        );
    }
    Ok(())
}

// Since the limits are based on `String::len`, which returns
// the number of bytes, we need to truncate the UTF-8 strings at the
// correct byte boundaries.
//
// This can be simplified once https://github.com/rust-lang/rust/issues/93743
// is stabilized.
#[inline]
pub fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        s.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = s.as_bytes()[lower_bound..=index]
            .iter()
            .rposition(|b| is_utf8_char_boundary(*b));

        // SAFETY: we know that the character boundary will be within four bytes
        lower_bound + new_index.unwrap()
    }
}

#[inline]
pub(crate) const fn is_utf8_char_boundary(b: u8) -> bool {
    // This is bit magic equivalent to: b < 128 || b >= 192
    (b as i8) >= -0x40
}

// Truncates a utf-8 string to the nearest character boundary,
// not exceeding max_bytes
fn truncate(s: &str, max_bytes: usize) -> &str {
    let closest = floor_char_boundary(s, max_bytes);

    &s[..closest]
}

mod tests {

    #[test]
    fn truncation() {
        use super::truncate;
        let s = "Hello, world!";

        assert_eq!(truncate(s, 5), "Hello");

        let s = "❤️🧡💛💚💙💜";
        assert_eq!(s.len(), 26);
        assert_eq!("❤".len(), 3);

        assert_eq!(truncate(s, 2), "");
        assert_eq!(truncate(s, 3), "❤");
        assert_eq!(truncate(s, 4), "❤");
    }
}
