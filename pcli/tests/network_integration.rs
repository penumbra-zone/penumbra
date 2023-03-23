//! Basic integration testing of `pcli` versus a target testnet.
//!
//! These tests are marked with `#[ignore]`, but can be run with:
//! `cargo test --package pcli -- --ignored --test-threads 1`
//!
//! Tests against the network in the `PENUMBRA_NODE_PD_URL` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributed 1cube.

use std::path::PathBuf;
use std::thread;

use assert_cmd::Command;
use directories::UserDirs;
//use once_cell::sync::Lazy;
use penumbra_component::stake::validator::ValidatorToml;
use predicates::prelude::*;
use regex::Regex;
use serde_json::Value;
use tempfile::{tempdir, NamedTempFile, TempDir};

use penumbra_chain::test_keys::{ADDRESS_0_STR, ADDRESS_1_STR, SEED_PHRASE};

const TEST_ASSET: &str = "1cube";

// The maximum amount of time any command is allowed to take before we error.
const TIMEOUT_COMMAND_SECONDS: u64 = 20;

// The time to wait before attempting to perform an undelegation claim.
/*
const UNBONDING_DURATION: Lazy<Duration> = Lazy::new(|| {
    let seconds = std::env::var("EPOCH_DURATION")
        .unwrap_or("100".to_string())
        .parse()
        .unwrap();
    Duration::from_secs(seconds)
});
 */

/// Import the wallet from seed phrase into a temporary directory.
fn load_wallet_into_tmpdir() -> TempDir {
    let tmpdir = tempdir().unwrap();

    let mut setup_cmd = Command::cargo_bin("pcli").unwrap();
    setup_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "keys",
            "import",
            "phrase",
            SEED_PHRASE,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    setup_cmd
        .assert()
        .stdout(predicate::str::contains("Saving backup wallet"));

    tmpdir
}

/// Look up a currently active validator on the testnet.
/// Will return the most bonded, which means the Penumbra Labs CI validator.
fn get_validator() -> String {
    let tmpdir = load_wallet_into_tmpdir();
    let mut validator_cmd = Command::cargo_bin("pcli").unwrap();
    validator_cmd
        .args([
            "--data-path",
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

#[ignore]
#[test]
fn transaction_send_from_addr_0_to_addr_1() {
    let tmpdir = load_wallet_into_tmpdir();

    // Send to self: tokens were distributed to `TEST_ADDRESS_0`, in our test
    // we'll send `TEST_ASSET` to `TEST_ADDRESS_1` and then check our balance.
    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            ADDRESS_1_STR,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    send_cmd.assert().success();

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    // The 1 is the index of the address which should be separated from the
    // test_asset only by whitespace.
    balance_cmd
        .assert()
        .stdout(predicate::str::is_match(format!(r"1\s*{TEST_ASSET}")).unwrap());

    // Cleanup: Send the asset back at the end of the test such that other tests begin
    // from the original state.
    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            ADDRESS_0_STR,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
}

#[ignore]
#[test]
fn transaction_sweep() {
    let tmpdir = load_wallet_into_tmpdir();

    let mut sweep_cmd = Command::cargo_bin("pcli").unwrap();
    sweep_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "sweep",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sweep_cmd.assert().success();
}

#[ignore]
#[test]
fn delegate_and_undelegate() {
    let tmpdir = load_wallet_into_tmpdir();

    // Get a validator from the testnet.
    let validator = get_validator();

    // Delegate a tiny bit of penumbra to the validator.
    let mut delegate_cmd = Command::cargo_bin("pcli").unwrap();
    delegate_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "delegate",
            "1penumbra",
            "--to",
            validator.as_str(),
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    delegate_cmd.assert().success();

    // Check we have some of the delegation token for that validator now.
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    balance_cmd
        .assert()
        .stdout(predicate::str::is_match(validator.as_str()).unwrap());

    // Now undelegate. We attempt `num_attempts` times in case an epoch boundary passes
    // while we prepare the delegation. See issues #1522, #2047.
    let num_attempts = 5;
    for _ in 0..num_attempts {
        let amount_to_undelegate = format!("0.99delegation_{}", validator.as_str());
        let mut undelegate_cmd = Command::cargo_bin("pcli").unwrap();
        undelegate_cmd
            .args([
                "--data-path",
                tmpdir.path().to_str().unwrap(),
                "tx",
                "undelegate",
                amount_to_undelegate.as_str(),
            ])
            .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
        let undelegation_result = undelegate_cmd.assert().try_success();

        // If the undelegation command succeeded, we can exit this loop.
        if undelegation_result.is_ok() {
            break;
        }
    }

    // Wait for the epoch duration.
    //thread::sleep(*UNBONDING_DURATION);
    // TODO: exercise undelegation claims.

    // Now sync.
    let mut sync_cmd = Command::cargo_bin("pcli").unwrap();
    sync_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "sync",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sync_cmd.assert().success();
}

#[ignore]
#[test]
fn swap() {
    let tmpdir = load_wallet_into_tmpdir();

    // Swap 1penumbra for some gn.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "--into",
            "gn",
            "1penumbra",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd.assert().success();

    // HACK: remove once #1749 is fixed
    thread::sleep(std::time::Duration::from_secs(10));

    // Cleanup: Swap the gn back (will fail if we received no gn in the above swap).
    let mut swap_back_cmd = Command::cargo_bin("pcli").unwrap();
    swap_back_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "--into",
            "penumbra",
            "0.9gn",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_back_cmd.assert().success();
}

#[ignore]
#[test]
fn governance_submit_proposal() {
    let tmpdir = load_wallet_into_tmpdir();

    // Get template for signaling proposal.
    let mut template_cmd = Command::cargo_bin("pcli").unwrap();
    template_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "template",
            "signaling",
            "--file",
            "proposal.toml",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    template_cmd.assert().success();

    // Submit signaling proposal.
    let mut submit_cmd = Command::cargo_bin("pcli").unwrap();
    submit_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "submit",
            "--file",
            "proposal.toml",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    submit_cmd.assert().success();

    // Now list the proposals.
    let mut proposals_cmd = Command::cargo_bin("pcli").unwrap();
    proposals_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "query",
            "governance",
            "list-proposals",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    proposals_cmd.assert().success();
}

#[ignore]
#[test]
fn duplicate_consensus_key_forbidden() {
    // Look up validator, so we have known-good data to munge.
    let validator = get_validator();
    let tmpdir = load_wallet_into_tmpdir();
    let mut query_cmd = Command::cargo_bin("pcli").unwrap();
    query_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "query",
            "validator",
            "definition",
            validator.as_str(),
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    query_cmd.assert().success();
    let validator_def_vec = query_cmd.unwrap().stdout;
    let original_validator_def: ValidatorToml =
        toml::from_str(&String::from_utf8_lossy(&validator_def_vec))
            .expect("can parse validator template as TOML");

    // Get template for promoting our node to validator.
    let mut template_cmd = Command::cargo_bin("pcli").unwrap();
    template_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "validator",
            "definition",
            "template",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    template_cmd.assert().success();
    let template_vec = template_cmd.unwrap().stdout;
    let mut new_validator_def: ValidatorToml =
        toml::from_str(&String::from_utf8_lossy(&template_vec))
            .expect("can parse validator template as TOML");

    // Overwrite randomly generated consensus key with one taken from
    // a real validator.
    new_validator_def.consensus_key = original_validator_def.consensus_key;

    // Write out new, intentionally broken validator definition.
    let validator_filepath = NamedTempFile::new().unwrap();
    std::fs::write(
        &validator_filepath,
        toml::to_string_pretty(&new_validator_def)
            .expect("Could not marshall new validator config as TOML"),
    )
    .expect("Could not overwrite validator config file with new definition");

    // Submit (intentionally broken) validator definition.
    let mut submit_cmd = Command::cargo_bin("pcli").unwrap();
    submit_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "validator",
            "definition",
            "upload",
            "--file",
            validator_filepath.path().to_str().unwrap(),
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    submit_cmd.assert().failure();
}

#[ignore]
#[test]
/// Ensures that attempting to modify an existing validator's consensus key fails.
fn mismatched_consensus_key_update_fails() {
    // Get template for promoting our node to validator.
    // We use a named tempfile so we can get a filepath for pcli cli.
    let validator_filepath = NamedTempFile::new().unwrap();
    let tmpdir = load_wallet_into_tmpdir();
    let mut template_cmd = Command::cargo_bin("pcli").unwrap();
    template_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "validator",
            "definition",
            "template",
            "--file",
            (validator_filepath.path().to_str().unwrap()),
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    template_cmd.assert().success();
    let template_content = std::fs::read_to_string(&validator_filepath)
        .expect("Could not read initial validator config file");
    let mut new_validator_def: ValidatorToml = toml::from_str(&template_content)
        .expect("Could not parse initial validator template as TOML");

    // Now we retrieve the actual tendermint consensus key from the testnet data dir.
    // Doing so assumes that the testnet-generated data was previously but in place,
    // which is a reasonable assumption in the context of running smoketest suite.
    let userdir = UserDirs::new().unwrap();
    let homedir = userdir
        .home_dir()
        .as_os_str()
        .to_str()
        .expect("Could not find home directory");
    let tm_key_filepath: PathBuf = [
        homedir,
        ".penumbra",
        "testnet_data",
        "node0",
        "tendermint",
        "config",
        "priv_validator_key.json",
    ]
    .iter()
    .collect();
    let tm_key_config: Value =
        serde_json::from_str(&std::fs::read_to_string(tm_key_filepath).unwrap())
            .expect("Could not read tendermint key config file");
    let tm_key: tendermint::PublicKey =
        serde_json::value::from_value(tm_key_config["pub_key"].clone())
            .expect("Could not parse tendermint key config file");

    // Modify initial validator definition template to use actual tm key.
    new_validator_def.consensus_key = tm_key;
    // Mark validator definition as "active".
    new_validator_def.enabled = true;
    // We used the validator identity in a previous test,
    // so set the template's sequence number to be higher.
    new_validator_def.sequence_number = 1000;

    // Write out revised (and incorrect!) validator definition.
    std::fs::write(
        &validator_filepath,
        toml::to_string_pretty(&new_validator_def)
            .expect("Could not marshall revised validator config as TOML"),
    )
    .expect("Could not overwrite validator config file with revised definition");

    // Run by itself, this test would need to munge the validator
    // definition and submit twice, once to create the validator,
    // and a second time to POST known-bad data. In the context
    // of the single-threaded smoketest suite, however, we previously
    // created a validator in [duplicate_consensus_key_forbidden].
    // Here, we reuse that validator's existence on the test-only chain
    // to confirm that subsequent validator updates fail.
    let mut resubmit_cmd = Command::cargo_bin("pcli").unwrap();
    resubmit_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "validator",
            "definition",
            "upload",
            "--file",
            validator_filepath.path().to_str().unwrap(),
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    // Ensure that command fails.
    resubmit_cmd.assert().failure();
}
