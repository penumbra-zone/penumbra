//! Contains functions related to the migration script of Testnet78.
#![allow(dead_code)]
use anyhow::Context;
use cnidarium::StateRead;
use cnidarium::{Snapshot, StateDelta, StateWrite, Storage};
use futures::TryStreamExt as _;
use futures::{pin_mut, StreamExt};
use jmt::RootHash;
use penumbra_sdk_app::app::StateReadExt as _;
use penumbra_sdk_dex::component::{PositionManager, StateReadExt, StateWriteExt};
use penumbra_sdk_dex::lp::position;
use penumbra_sdk_dex::lp::position::Position;
use penumbra_sdk_governance::proposal_state::State as ProposalState;
use penumbra_sdk_governance::Proposal;
use penumbra_sdk_governance::StateReadExt as _;
use penumbra_sdk_governance::StateWriteExt as _;
use penumbra_sdk_proto::core::component::governance::v1 as pb_governance;
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_sct::component::clock::EpochManager;
use penumbra_sdk_sct::component::clock::EpochRead;
use penumbra_sdk_stake::validator::Validator;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tendermint_config::TendermintConfig;
use tracing::instrument;

use crate::network::generate::NetworkConfig;

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
/// - Close and re-open all *open* positions so that they are re-indexed.
/// - Update DEX parameters
#[instrument]
pub async fn migrate(
    storage: Storage,
    pd_home: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    /* `Migration::prepare`: collect basic migration data, logging, initialize alt-storage if needed */
    let initial_state = storage.latest_snapshot();

    let chain_id = initial_state.get_chain_id().await?;
    let root_hash = initial_state
        .root_hash()
        .await
        .expect("chain state has a root hash");

    let pre_upgrade_height = initial_state
        .get_block_height()
        .await
        .expect("chain state has a block height");
    let post_upgrade_height = pre_upgrade_height.wrapping_add(1);

    let pre_upgrade_root_hash: RootHash = root_hash.into();

    /* `Migration::migrate`: reach into the chain state and perform an offline state transition */
    let mut delta = StateDelta::new(initial_state);

    let (migration_duration, post_upgrade_root_hash) = {
        let start_time = std::time::SystemTime::now();

        // Migrate empty (deleted) packet commitments to be deleted at the tree level
        delete_empty_deleted_packet_commitments(&mut delta).await?;

        // Adjust the length of `Validator` fields.
        truncate_validator_fields(&mut delta).await?;

        // Adjust the length of governance proposal fields.
        truncate_proposal_fields(&mut delta).await?;

        // Adjust the length of governance proposal outcome fields.
        truncate_proposal_outcome_fields(&mut delta).await?;

        // Write the new dex parameters (with the execution budget field) to the state.
        update_dex_params(&mut delta).await?;

        // Re-index all open positions.
        reindex_dex_positions(&mut delta).await?;

        // Reset the application height and halt flag.
        delta.ready_to_start();
        delta.put_block_height(0u64);

        // Finally, commit the changes to the chain state.
        let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
        tracing::info!(?post_upgrade_root_hash, "post-migration root hash");

        (
            start_time.elapsed().expect("start is set"),
            post_upgrade_root_hash,
        )
    };

    tracing::info!("migration completed, generating genesis and signing state...");

    /* `Migration::complete`: the state transition has been performed, we prepare the checkpointed genesis and signing state */
    let app_state = penumbra_sdk_app::genesis::Content {
        chain_id,
        ..Default::default()
    };
    let mut genesis = NetworkConfig::make_genesis(app_state.clone()).expect("can make genesis");
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

    tracing::info!("generating checkpointed genesis");
    let checkpoint = post_upgrade_root_hash.0.to_vec();
    let genesis = NetworkConfig::make_checkpoint(genesis, Some(checkpoint));

    tracing::info!("writing genesis to disk");
    let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
    tracing::info!("genesis: {}", genesis_json);
    let genesis_path = pd_home.join("genesis.json");
    std::fs::write(genesis_path, genesis_json).expect("can write genesis");

    tracing::info!("updating signing state");
    let validator_state_path = pd_home.join("priv_validator_state.json");
    let fresh_validator_state = crate::network::generate::NetworkValidator::initial_state();
    std::fs::write(validator_state_path, fresh_validator_state).expect("can write validator state");

    tracing::info!(
        pre_upgrade_height,
        post_upgrade_height,
        ?pre_upgrade_root_hash,
        ?post_upgrade_root_hash,
        duration = migration_duration.as_secs(),
        "migration fully complete"
    );

    Ok(())
}

async fn delete_empty_deleted_packet_commitments(
    delta: &mut StateDelta<Snapshot>,
) -> anyhow::Result<()> {
    let prefix_key = "ibc-data/commitments/";
    let stream = delta.prefix_raw(&prefix_key);

    pin_mut!(stream);

    while let Some(entry) = stream.next().await {
        let (key, value) = entry?;
        if value.is_empty() {
            delta.delete(key);
        }
    }

    Ok(())
}

/// Write the updated dex parameters to the chain state.
async fn update_dex_params(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let mut dex_params = delta
        .get_dex_params()
        .await
        .expect("chain state is initialized");
    dex_params.max_execution_budget = 64;
    delta.put_dex_params(dex_params);

    Ok(())
}

/// Reindex opened liquidity positions to augment the price indexes.
async fn reindex_dex_positions(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    tracing::info!("running dex re-indexing migration");
    let prefix_key_lp = penumbra_sdk_dex::state_key::all_positions();
    let stream_all_lp = delta.prefix::<Position>(&prefix_key_lp);
    let stream_open_lp = stream_all_lp.filter_map(|entry| async {
        match entry {
            Ok((_, lp)) if lp.state == position::State::Opened => Some(lp),
            _ => None,
        }
    });
    pin_mut!(stream_open_lp);

    while let Some(lp) = stream_open_lp.next().await {
        // Re-hash the position, since the key is a bech32 string.
        let id = lp.id();
        // Close the position, adjusting all its index entries.
        delta.close_position_by_id(&id).await?;
        // Erase the position from the state, so that we circumvent the `update_position` guard.
        delta.delete(penumbra_sdk_dex::state_key::position_by_id(&id));
        // Open a position with the adjusted indexing logic.
        delta.open_position(lp).await?;
    }
    tracing::info!("completed dex migration");
    Ok(())
}

///   * Validators:
///    - `name` (140 bytes)
///    - `website` (70 bytes)
///    - `description` (280 bytes)
async fn truncate_validator_fields(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    tracing::info!("truncating validator fields");
    let key_prefix_validators = penumbra_sdk_stake::state_key::validators::definitions::prefix();
    let all_validators = delta
        .prefix_proto::<penumbra_sdk_proto::core::component::stake::v1::Validator>(
            &key_prefix_validators,
        )
        .try_collect::<Vec<(
            String,
            penumbra_sdk_proto::core::component::stake::v1::Validator,
        )>>()
        .await?;

    for (key, mut validator) in all_validators {
        validator.name = truncate(&validator.name, 140).to_string();
        validator.website = truncate(&validator.website, 70).to_string();
        validator.description = truncate(&validator.description, 280).to_string();

        // Ensure the validator can be serialized back to the domain type:
        let validator: Validator = validator.try_into()?;
        tracing::info!("put key {:?}", key);
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
    tracing::info!("truncating proposal fields");
    let next_proposal_id: u64 = delta.next_proposal_id().await?;

    // Range each proposal and truncate the fields.
    for proposal_id in 0..next_proposal_id {
        tracing::info!("truncating proposal: {}", proposal_id);
        let proposal = delta
            .get_proto::<pb_governance::Proposal>(
                &penumbra_sdk_governance::state_key::proposal_definition(proposal_id),
            )
            .await?;

        if proposal.is_none() {
            break;
        }

        let mut proposal = proposal.unwrap();

        proposal.title = truncate(&proposal.title, 80).to_string();
        proposal.description = truncate(&proposal.description, 10_000).to_string();

        // Depending on the proposal type, we may need to truncate additional fields.
        match proposal
            .payload
            .clone()
            .expect("proposal payload always set")
        {
            pb_governance::proposal::Payload::Signaling(commit) => {
                proposal.payload = Some(pb_governance::proposal::Payload::Signaling(
                    pb_governance::proposal::Signaling {
                        commit: truncate(&commit.commit, 255).to_string(),
                    },
                ));
            }
            pb_governance::proposal::Payload::Emergency(_halt_chain) => {}
            pb_governance::proposal::Payload::ParameterChange(mut param_change) => {
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

                proposal.payload = Some(pb_governance::proposal::Payload::ParameterChange(
                    param_change,
                ));
            }
            pb_governance::proposal::Payload::CommunityPoolSpend(_transaction_plan) => {}
            pb_governance::proposal::Payload::UpgradePlan(_height) => {}
            pb_governance::proposal::Payload::FreezeIbcClient(client_id) => {
                proposal.payload = Some(pb_governance::proposal::Payload::FreezeIbcClient(
                    pb_governance::proposal::FreezeIbcClient {
                        client_id: truncate(&client_id.client_id, 128).to_string(),
                    },
                ));
            }
            pb_governance::proposal::Payload::UnfreezeIbcClient(client_id) => {
                proposal.payload = Some(pb_governance::proposal::Payload::UnfreezeIbcClient(
                    pb_governance::proposal::UnfreezeIbcClient {
                        client_id: truncate(&client_id.client_id, 128).to_string(),
                    },
                ));
            }
        };

        // Store the truncated proposal data
        tracing::info!(
            "put key {:?}",
            penumbra_sdk_governance::state_key::proposal_definition(proposal_id)
        );
        // Ensure the proposal can be serialized back to the domain type:
        let proposal: Proposal = proposal.try_into()?;
        delta.put(
            penumbra_sdk_governance::state_key::proposal_definition(proposal_id),
            proposal,
        );
    }

    Ok(())
}

///   * Governance Proposal Withdrawals:
///    - `reason` (1024 bytes)
async fn truncate_proposal_outcome_fields(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    tracing::info!("truncating proposal outcome fields");
    let next_proposal_id: u64 = delta.next_proposal_id().await?;

    // Range each proposal outcome and truncate the fields.
    for proposal_id in 0..next_proposal_id {
        tracing::info!("truncating proposal outcomes: {}", proposal_id);
        let proposal_state = delta
            .get_proto::<pb_governance::ProposalState>(
                &penumbra_sdk_governance::state_key::proposal_state(proposal_id),
            )
            .await?;

        if proposal_state.is_none() {
            break;
        }

        let mut proposal_state = proposal_state.unwrap();

        match proposal_state
            .state
            .clone()
            .expect("proposal state always set")
        {
            pb_governance::proposal_state::State::Withdrawn(reason) => {
                proposal_state.state = Some(pb_governance::proposal_state::State::Withdrawn(
                    pb_governance::proposal_state::Withdrawn {
                        reason: truncate(&reason.reason, 1024).to_string(),
                    },
                ));
            }
            pb_governance::proposal_state::State::Voting(_) => {}
            pb_governance::proposal_state::State::Finished(ref outcome) => match outcome
                .outcome
                .clone()
                .expect("proposal outcome always set")
                .outcome
                .expect("proposal outcome always set")
            {
                pb_governance::proposal_outcome::Outcome::Passed(_) => {}
                pb_governance::proposal_outcome::Outcome::Failed(withdrawn) => {
                    match withdrawn.withdrawn {
                        None => {
                            // Withdrawn::No
                        }
                        Some(pb_governance::proposal_outcome::Withdrawn { reason }) => {
                            // Withdrawn::WithReason
                            proposal_state.state =
                                Some(pb_governance::proposal_state::State::Finished(
                                    pb_governance::proposal_state::Finished {
                                        outcome: Some(pb_governance::ProposalOutcome{
                                            outcome: Some(pb_governance::proposal_outcome::Outcome::Failed(
                                                pb_governance::proposal_outcome::Failed {
                                                    withdrawn:
                                                        Some(pb_governance::proposal_outcome::Withdrawn {
                                                            reason: truncate(&reason, 1024)
                                                                .to_string(),
                                                        }),
                                                },
                                            )),
                                        }),
                                    },
                                ));
                        }
                    }
                }
                pb_governance::proposal_outcome::Outcome::Slashed(withdrawn) => {
                    match withdrawn.withdrawn {
                        None => {
                            // Withdrawn::No
                        }
                        Some(pb_governance::proposal_outcome::Withdrawn { reason }) => {
                            // Withdrawn::WithReason
                            proposal_state.state = Some(pb_governance::proposal_state::State::Finished(
                                    pb_governance::proposal_state::Finished {
                                        outcome: Some(pb_governance::ProposalOutcome{
                                            outcome: Some(pb_governance::proposal_outcome::Outcome::Slashed(
                                                pb_governance::proposal_outcome::Slashed {
                                                    withdrawn:
                                                        Some(pb_governance::proposal_outcome::Withdrawn {
                                                            reason: truncate(&reason, 1024)
                                                                .to_string(),
                                                        }),
                                                },
                                            )),
                                        }),
                                    },
                                ));
                        }
                    }
                }
            },
            pb_governance::proposal_state::State::Claimed(ref outcome) => match outcome
                .outcome
                .clone()
                .expect("outcome is set")
                .outcome
                .expect("outcome is set")
            {
                pb_governance::proposal_outcome::Outcome::Passed(_) => {}
                pb_governance::proposal_outcome::Outcome::Failed(withdrawn) => {
                    match withdrawn.withdrawn {
                        None => {
                            // Withdrawn::No
                        }
                        Some(pb_governance::proposal_outcome::Withdrawn { reason }) => {
                            // Withdrawn::WithReason
                            proposal_state.state = Some(pb_governance::proposal_state::State::Claimed(
                                    pb_governance::proposal_state::Claimed {
                                        outcome: Some(pb_governance::ProposalOutcome{
                                            outcome: Some(pb_governance::proposal_outcome::Outcome::Failed(
                                                pb_governance::proposal_outcome::Failed{
                                                    withdrawn:
                                                        Some(pb_governance::proposal_outcome::Withdrawn {
                                                            reason: truncate(&reason, 1024)
                                                                .to_string(),
                                                        }),
                                                },
                                            )),
                                        }),
                                    },
                                ));
                        }
                    }
                }
                pb_governance::proposal_outcome::Outcome::Slashed(withdrawn) => {
                    match withdrawn.withdrawn {
                        None => {
                            // Withdrawn::No
                        }
                        Some(pb_governance::proposal_outcome::Withdrawn { reason }) => {
                            proposal_state.state = Some(pb_governance::proposal_state::State::Claimed(
                                    pb_governance::proposal_state::Claimed {
                                        outcome: Some(pb_governance::ProposalOutcome{
                                            outcome: Some(pb_governance::proposal_outcome::Outcome::Slashed(
                                                pb_governance::proposal_outcome::Slashed{
                                                    withdrawn:
                                                        Some(pb_governance::proposal_outcome::Withdrawn {
                                                            reason: truncate(&reason, 1024)
                                                                .to_string(),
                                                        }),
                                                },
                                            )),
                                        }),
                                    },
                                ));
                        }
                    }
                }
            },
        }

        // Store the truncated proposal state data
        tracing::info!(
            "put key {:?}",
            penumbra_sdk_governance::state_key::proposal_state(proposal_id)
        );
        let proposal_state: ProposalState = proposal_state.try_into()?;
        delta.put(
            penumbra_sdk_governance::state_key::proposal_state(proposal_id),
            proposal_state,
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

/// Edit the node's CometBFT config file to set two values:
///
///   * mempool.max_tx_bytes
///   * mempool.max_txs_bytes
///
/// These values will affect consensus, but the config settings are specified for CometBFT
/// specifically.
#[instrument]
pub(crate) fn update_cometbft_mempool_settings(cometbft_home: PathBuf) -> anyhow::Result<()> {
    let cometbft_config_path = cometbft_home.join("config").join("config.toml");
    tracing::debug!(cometbft_config_path = %cometbft_config_path.display(), "opening cometbft config file");
    let mut cometbft_config = TendermintConfig::load_toml_file(&cometbft_config_path)
        .context("failed to load pre-migration cometbft config file")?;
    // The new values were updated in GH4594 & GH4632.
    let desired_max_txs_bytes = 10485760;
    let desired_max_tx_bytes = 30720;
    // Set new value
    cometbft_config.mempool.max_txs_bytes = desired_max_txs_bytes;
    cometbft_config.mempool.max_tx_bytes = desired_max_tx_bytes;
    // Overwrite file
    let mut fh = OpenOptions::new()
        .create(false)
        .write(true)
        .truncate(true)
        .open(cometbft_config_path.clone())
        .context("failed to open cometbft config file for writing")?;
    fh.write_all(toml::to_string(&cometbft_config)?.as_bytes())
        .context("failed to write updated cometbft config to toml file")?;
    Ok(())
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
