//! Basic integration testing of `pcli` versus a target testnet.
//!
//! These tests are marked with `#[ignore]`, but can be run with:
//! `cargo test --package pcli -- --ignored --test-threads 1`
//!
//! Tests against the network in the `PENUMBRA_NODE_HOSTNAME` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributed 1cube.

use std::{thread, time};

use assert_cmd::Command;
use predicates::prelude::*;
use regex::Regex;
use tempfile::{tempdir, TempDir};

// This address is for test purposes, allocations were added beginning with
// the 016-Pandia testnet.
const TEST_SEED_PHRASE: &str = "benefit cherry cannon tooth exhibit law avocado spare tooth that amount pumpkin scene foil tape mobile shine apology add crouch situate sun business explain";

// These addresses both correspond to the test wallet above.
const TEST_ADDRESS_0: &str = "penumbrav2t13vh0fkf3qkqjacpm59g23ufea9n5us45e4p5h6hty8vg73r2t8g5l3kynad87u0n9eragf3hhkgkhqe5vhngq2cw493k48c9qg9ms4epllcmndd6ly4v4dw2jcnxaxzjqnlvnw";
const TEST_ADDRESS_1: &str = "penumbrav2t1gl609fq6xzjcqn3hz3crysw2s0nkt330lyhaq403ztmrm3yygsgdklt9uxfs0gedwp6sypp5k5ln9t62lvs9t0a990q832wnxak8r939g5u6uz5aessd8saxvv7ewlz4hhqnws";

const TEST_ASSET: &str = "1cube";

const BLOCK_TIME_SECONDS: u64 = 10;
// We need to wait for syncing to occur.
const TIMEOUT_COMMAND_SECONDS: u64 = 360;

/// Import the wallet from seed phrase into a temporary directory.
fn load_wallet_into_tmpdir() -> TempDir {
    let tmpdir = tempdir().unwrap();

    let mut setup_cmd = Command::cargo_bin("pcli").unwrap();
    setup_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "keys",
            "import",
            "phrase",
            TEST_SEED_PHRASE,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    setup_cmd
        .assert()
        .stdout(predicate::str::contains("Saving backup wallet"));

    tmpdir
}

#[ignore]
#[test]
fn transaction_send_from_addr_0_to_addr_1() {
    let tmpdir = load_wallet_into_tmpdir();

    // Send to self: tokens were distributed to `TEST_ADDRESS_0`, in our test
    // we'll send `TEST_ASSET` to `TEST_ADDRESS_1` and then check our balance.
    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            TEST_ADDRESS_1,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    send_cmd.assert().success();

    // Wait for a couple blocks for the transaction to be confirmed.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
            "--by-address",
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
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            TEST_ADDRESS_0,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    // Wait for a couple blocks for the transaction to be confirmed before doing other tests.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);
}

#[ignore]
#[test]
fn transaction_sweep() {
    let tmpdir = load_wallet_into_tmpdir();

    let mut sweep_cmd = Command::cargo_bin("pcli").unwrap();
    sweep_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "sweep",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sweep_cmd.assert().success();

    // Wait for a couple blocks for the transaction to be confirmed before doing other tests.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);
}

#[ignore]
#[test]
fn delegate_and_undelegate() {
    let tmpdir = load_wallet_into_tmpdir();

    // Get the list of validators.
    let mut validator_cmd = Command::cargo_bin("pcli").unwrap();
    validator_cmd
        .args(&[
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
    let validator = captures.unwrap()[0].to_string();

    // Delegate a tiny bit of penumbra to the validator.
    let mut delegate_cmd = Command::cargo_bin("pcli").unwrap();
    delegate_cmd
        .args(&[
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

    // Wait for a couple blocks for the transaction to be confirmed.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);

    // Check we have some of the delegation token for that validator now.
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    balance_cmd
        .assert()
        .stdout(predicate::str::is_match(validator.as_str()).unwrap());

    // Now undelegate.
    let amount_to_undelegate = format!("0.99delegation_{}", validator.as_str());
    let mut undelegate_cmd = Command::cargo_bin("pcli").unwrap();
    undelegate_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "undelegate",
            amount_to_undelegate.as_str(),
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    undelegate_cmd.assert().success();

    // Wait for a couple blocks for the transaction to be confirmed before doing other tests.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);
}

#[ignore]
#[test]
fn swap() {
    let tmpdir = load_wallet_into_tmpdir();

    // Swap 1penumbra for some gn.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args(&[
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

    // Wait for a couple blocks for the transaction to be confirmed.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);

    // Cleanup: Swap the gn back (will fail if we received no gn in the above swap).
    let mut swap_back_cmd = Command::cargo_bin("pcli").unwrap();
    swap_back_cmd
        .args(&[
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

    // Wait for a couple blocks for the transaction to be confirmed before doing other tests.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);
}

#[ignore]
#[test]
fn governance_submit_proposal() {
    let tmpdir = load_wallet_into_tmpdir();

    // Get template for signaling proposal.
    let mut template_cmd = Command::cargo_bin("pcli").unwrap();
    template_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "template",
            "--kind",
            "signaling",
            "--file",
            "proposal.json",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    template_cmd.assert().success();

    // Submit signaling proposal.
    let mut submit_cmd = Command::cargo_bin("pcli").unwrap();
    submit_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "submit",
            "--file",
            "proposal.json",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    submit_cmd.assert().success();

    // Wait for a couple blocks for the transaction to be confirmed.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);

    // Now list the proposals.
    let mut proposals_cmd = Command::cargo_bin("pcli").unwrap();
    proposals_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "query",
            "governance",
            "list-proposals",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    proposals_cmd.assert().success();
}
