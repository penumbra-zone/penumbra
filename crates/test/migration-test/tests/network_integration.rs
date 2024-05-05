#![allow(dead_code, unused_imports)]
//! Migration testing between Penumbra networks, for validating chain upgrades.
//!
//! These tests are marked with `#[migration-test]`, and so can be run with:
//! `cargo test --package migration-test --all-features -- --test-threads 1`
//!
//! Tests against the network in the `PENUMBRA_NODE_PD_URL` environment variable.
//!
//! Tests use the smoke test account. Tries to avoid assumptions about whether
//! or how many times the smoke-test suite has been run.
//!
//! See the latest testnet's `allocations.csv` for the initial allocations to the test validator addresses
//! ([`ADDRESS_0_STR`], [`ADDRESS_1_STR`]).

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::thread;
use std::{path::PathBuf, time::Duration};

use assert_cmd::Command;
use directories::UserDirs;
use once_cell::sync::Lazy;
use predicates::prelude::*;
use regex::Regex;
use tempfile::{tempdir, NamedTempFile, TempDir};

use tracing::instrument;

use penumbra_keys::test_keys::SEED_PHRASE;

// The number 50,000 is chosen to be greater than two-third of the default
// genesis stake for a devnet validator, which is 25,000.
const TEST_DELEGATION: &str = "50000penumbra";

// Amount to pay as deposit for submitting the halt proposal. See GH3455 for format.
const TEST_PROPOSAL_DEPOSIT: &str = "10penumbra";

// The maximum amount of time any command is allowed to take before we error.
const TIMEOUT_COMMAND_SECONDS: u64 = 20;

const PROPOSAL_VOTING_BLOCKS: Lazy<u64> = Lazy::new(|| {
    std::env::var("PROPOSAL_VOTING_BLOCKS")
        .unwrap_or("50".to_string())
        .parse()
        .unwrap()
});

// The length of wall time that constitutes an epoch. Used to sleep
// so that we're guaranteed to advance to the subsequent epoch.
static EPOCH_DURATION: Lazy<Duration> = Lazy::new(|| {
    let blocks: f64 = std::env::var("EPOCH_DURATION")
        .unwrap_or("50".to_string())
        .parse()
        .unwrap();
    // 0.5 -> 0.6 for comfort, since 500ms is only an estimate.
    Duration::from_secs((0.6 * blocks) as u64)
});

// Assume we're running against local devnet.
static PD_NODE_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("PENUMBRA_NODE_PD_URL")
        .unwrap_or("http://localhost:8080".to_string())
        .parse()
        .unwrap()
});

/// Build `Command` for `pcli`. By default, uses the `pcli` from the current workspace,
/// but can be overridden via `TEST_PCLI_PATH` env var to use a discrete binary on disk.
/// The latter case is useful for migration testing, running a previously released
/// version of pcli for a pre-upgrade devnet.
///
/// Important: the `TEST_PCLI_PATH` env var should be *relative to the repo root*.
fn get_pcli_path() -> Command {
    match std::env::var("TEST_PCLI_PATH") {
        Ok(p) => {
            let git_output = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .unwrap()
                .stdout;
            let repo_root = std::str::from_utf8(&git_output).unwrap().trim();
            let fullpath = Path::new(&repo_root).join(&p);
            assert!(
                fullpath.is_absolute(),
                "fullpath to pcli must be absolute: '{}'",
                fullpath.display()
            );
            assert!(
                fullpath.is_file(),
                "fullpath to pcli must exist: '{}'",
                fullpath.display()
            );
            Command::new(fullpath)
        }
        Err(_) => Command::cargo_bin("pcli").unwrap(),
    }
}

/// Import the wallet from seed phrase into a temporary directory.
fn load_wallet_into_tmpdir() -> TempDir {
    let tmpdir = tempdir().unwrap();
    let mut setup_cmd = get_pcli_path();
    setup_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "init",
            "--grpc-url",
            &PD_NODE_URL,
            "soft-kms",
            "import-phrase",
        ])
        .write_stdin(SEED_PHRASE)
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    setup_cmd
        .assert()
        .stdout(predicate::str::contains("Writing generated config"));
    tmpdir
}

/// Sync the wallet. Fallible, so we can use it so confirm network is halted.
fn sync(tmpdir: &TempDir) -> anyhow::Result<()> {
    let mut sync_cmd = get_pcli_path();
    sync_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "sync"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sync_cmd.assert().try_success()?;
    Ok(())
}

/// Look up a currently active validator on the testnet.
/// Will return the most bonded, which means the Penumbra Labs CI validator.
fn get_validator(tmpdir: &TempDir) -> String {
    let mut validator_cmd = get_pcli_path();
    validator_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "query",
            "validator",
            "list",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    validator_cmd.assert().success();

    // Pull out one of the validators from stdout.
    let stdout_vec = validator_cmd.unwrap().stdout;
    let validator_regex = Regex::new(r"penumbravalid1\w{58}").unwrap();
    let captures = validator_regex.captures(std::str::from_utf8(&stdout_vec).unwrap());

    // We retrieve the first match via index 0, which results in most trusted.
    captures.unwrap()[0].to_string()
}

/// Look up current block height on target network.
fn get_current_height(tmpdir: &TempDir) -> u64 {
    let mut info_cmd = get_pcli_path();
    info_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "query",
            "chain",
            "info",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    info_cmd.assert().success();

    let stdout_vec = info_cmd.unwrap().stdout;
    let height_regex = Regex::new(r"Current Block Height\s+(\d+)").unwrap();
    let captures = height_regex.captures(std::str::from_utf8(&stdout_vec).unwrap());

    // Reminder: index 0 is entire match, index 1 is first captured group.
    captures.unwrap()[1]
        .parse()
        .expect("block height in pcli output is u64")
}

// Needed functions:
//
//   get_current_height()
//   generate_proposal
//   submit_proposal
//
//

/// Look up top validator and delegate `TEST_DELEGATION` to it.
#[cfg_attr(not(feature = "migration-test"), ignore)]
#[test]
fn delegate() {
    tracing_subscriber::fmt::try_init().ok();
    tracing::info!("delegate_and_undelegate");
    let tmpdir = load_wallet_into_tmpdir();

    // Get a validator from the testnet.
    let validator = get_validator(&tmpdir);

    // Delegate a tiny bit of penumbra to the validator.
    let mut delegate_cmd = get_pcli_path();
    delegate_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "delegate",
            &TEST_DELEGATION,
            "--to",
            validator.as_str(),
            // use dedicated subaccount for smoke delegations
            "--source",
            "2",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    tracing::info!(?delegate_cmd, "running delegation command");
    delegate_cmd.assert().success();
    // Wait for bonding, otherwise delegation isn't active.
    tracing::info!("waiting for stake to bonding...");
    thread::sleep(*EPOCH_DURATION);
}

/// Look up any delegation to a validator, and undelegate it.
/// Useful for "undoing" the delegate action, which can help with keeping the
/// test suite somewhat idempotent.
fn undelegate() {
    tracing::info!("check that we have some of the delegation token");
    let tmpdir = load_wallet_into_tmpdir();
    let validator = get_validator(&tmpdir);
    // Check we have some of the delegation token for that validator now.
    let mut balance_cmd = get_pcli_path();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    balance_cmd
        .assert()
        .stdout(predicate::str::is_match(validator.as_str()).unwrap());

    let balance_output = balance_cmd.output().unwrap().stdout;
    let balance_output_string = String::from_utf8_lossy(&balance_output);

    tracing::debug!(?balance_output_string, "balance output string");

    // We successfully delegated. But since the validator exchange rates are dynamic, we
    // need to pull the amount of delegation tokens we obtained so that we can later
    // try to execute an undelegation (`tx undelegate <AMOUNT><DELEGATION_TOKEN_DENOM>`).
    // To do this, we use a regex to extract the amount of delegation tokens we obtained:
    let delegation_token_pattern = Regex::new(r"(\d+\.?\d?m?delegation_[a-zA-Z0-9]*)").unwrap();
    let (delegation_token_str, [_match]) = delegation_token_pattern
        .captures(&balance_output_string)
        .expect("can find delegation token in balance output")
        .extract();

    tracing::info!("attempting undelegation");
    let mut undelegate_cmd = get_pcli_path();
    undelegate_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "undelegate",
            delegation_token_str,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    undelegate_cmd.assert().success();

    tracing::info!("undelegation succeeded, wait an epoch before claiming.");
    // Wait for the epoch duration.
    thread::sleep(*EPOCH_DURATION);
    tracing::info!("epoch passed, claiming now");
    let mut undelegate_claim_cmd = get_pcli_path();
    undelegate_claim_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "undelegate-claim",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    tracing::info!(?undelegate_claim_cmd, "claiming");
    undelegate_claim_cmd.assert().success();
    tracing::info!("success!");
    sync(&tmpdir).expect("sync is successful");
}

/// TOML for an "upgrade-plan" governance proposal.
// Intentionally avoiding importing this type to adhere to strict
// CLI interfacts for the pcli binaries.
#[derive(Deserialize, Serialize)]
struct UpgradePlan {
    id: u64,
    title: String,
    description: String,
    kind: String,
    height: u64,
}

#[cfg_attr(not(feature = "migration-test"), ignore)]
#[test]
#[instrument]
/// Create, submit, and vote on proposal for halting the chain.
/// Will automatically choose an upgrade height in the next epoch.
fn submit_chain_upgrade_proposal() {
    let tmpdir = load_wallet_into_tmpdir();
    // Get template for signaling proposal.
    let mut template_cmd = get_pcli_path();
    template_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "template",
            "upgrade-plan",
            "--file",
            "proposal.toml",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    tracing::info!("generating upgrade-plan proposal");

    template_cmd.assert().success();

    // Look up current height and calculate halt height.
    let current_height = get_current_height(&tmpdir);
    let upgrade_height = current_height + (*PROPOSAL_VOTING_BLOCKS * 2);
    // Wait a bit longer than necessary so we're sure the network will be halted.
    let blocks_to_wait: u64 = ((upgrade_height - current_height) as f64 * 1.2) as u64;

    let proposal_contents = std::fs::read_to_string("proposal.toml")
        .expect("could not read upgrade-plan proposal toml file");
    let mut proposal: UpgradePlan = toml::from_str(&proposal_contents)
        .expect("can parse upgrade-plan proposal toml file to UpgradePlan");

    proposal.height = upgrade_height;
    proposal.title = "migration-test".to_string();
    proposal.description = "automated proposal via migration-test suite".to_string();

    let proposal_path = Path::new("proposal.toml");
    std::fs::write(
        &proposal_path,
        toml::to_string_pretty(&proposal).expect("could not marshal upgrade-plan as TOML"),
    )
    .expect("can write upgrade-plan proposal to tempfile");

    // Submit upgrade-plan proposal.
    tracing::info!("submitting upgrade-plan proposal");
    let mut submit_cmd = get_pcli_path();
    submit_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "submit",
            "--file",
            &proposal_path.to_str().unwrap(),
            // use dedicated smoke subaccount for proposal voting
            "--source",
            "2",
            "--deposit-amount",
            TEST_PROPOSAL_DEPOSIT,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    submit_cmd.assert().success();

    // Vote on proposal.
    tracing::info!("voting yes on proposal {}", &proposal.id);
    let mut vote_cmd = get_pcli_path();
    vote_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "vote",
            "yes",
            "--on",
            &proposal.id.to_string(),
            // use dedicated subaccount for smoke delegations
            "--source",
            "2",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    vote_cmd.assert().success();

    // Wait for it to pass, after which chain should be halted
    tracing::warn!("waiting {} blocks for chain to halt...", blocks_to_wait);
    wait_n_blocks(blocks_to_wait);

    // Assert that a sync fails
    match sync(&tmpdir) {
        // OK is bad, since we expected the chain to be halted.
        Ok(_) => {
            assert!(false, "chain is still running, not halted");
        }
        // Error is expected and good, as it indicates chain has halted.
        Err(_) => {}
    }
}

/// Sleep for the specified number of blocks. Assumes blocktime of 500ms.
fn wait_n_blocks(blocks: u64) {
    // 0.5 -> 0.6 for comfort, since 500ms is only an estimate.
    let d = Duration::from_secs((0.6 * blocks as f64) as u64);
    thread::sleep(d);
}
