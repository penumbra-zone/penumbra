//! Basic integration testing of `pcli` versus a target testnet.
//!
//! These tests are marked with `#[ignore]`, but can be run with:
//! `cargo test --package pcli -- --ignored --test-threads 1`
//!
//! Tests against the network in the `PENUMBRA_NODE_PD_URL` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributed 20pusd ([`TEST_ASSET`]).
//!
//! See the latest testnet's `allocations.csv` for the initial allocations to the test validator addresses
//! ([`ADDRESS_0_STR`], [`ADDRESS_1_STR`]).

use std::thread;
use std::{path::PathBuf, time::Duration};

use assert_cmd::Command;
use directories::UserDirs;
use once_cell::sync::Lazy;
use penumbra_sdk_stake::validator::ValidatorToml;
use predicates::prelude::*;
use regex::Regex;
use serde_json::Value;
use tempfile::{tempdir, NamedTempFile, TempDir};
use url::Url;

use penumbra_sdk_keys::test_keys::{ADDRESS_0_STR, ADDRESS_1_STR, SEED_PHRASE};
use penumbra_sdk_proto::core::transaction::v1::TransactionView as ProtoTransactionView;
use penumbra_sdk_transaction::view::TransactionView;

// The number "1020" is chosen so that this is bigger than u64::MAX
// when accounting for the 10e18 scaling factor from the base denom.
const TEST_ASSET: &str = "1020test_usd";

// The maximum amount of time any command is allowed to take before we error.
const TIMEOUT_COMMAND_SECONDS: u64 = 20;

// The time to wait before attempting to perform an undelegation claim.
// The "unbonding_delay" value is specified in blocks, and in the smoke tests,
// block time is set to ~500ms, so we'll take the total number of blocks
// that must elapse and sleep half that many seconds.
static UNBONDING_DELAY: Lazy<Duration> = Lazy::new(|| {
    let blocks: f64 = std::env::var("UNBONDING_DELAY")
        .unwrap_or("50".to_string())
        .parse()
        .unwrap();
    // 0.5 -> 0.6 for comfort, since 500ms is only an estimate.
    Duration::from_secs((0.6 * blocks) as u64)
});

/// Import the wallet from seed phrase into a temporary directory.
fn load_wallet_into_tmpdir() -> TempDir {
    let tmpdir = tempdir().unwrap();

    let grpc_url: Url = std::env::var("PENUMBRA_NODE_PD_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned())
        .parse()
        .expect("failed to parse PENUMBRA_NODE_PD_URL");

    let mut setup_cmd = Command::cargo_bin("pcli").unwrap();
    setup_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "init",
            "--grpc-url",
            &grpc_url.to_string(),
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

#[allow(dead_code)]
fn load_string_to_file(content: String, tmpdir: &TempDir) -> NamedTempFile {
    let mut file = NamedTempFile::new_in(tmpdir.path()).unwrap();
    use std::io::Write;
    write!(file, "{}", content).unwrap();
    file
}

/// Sync the wallet.
fn sync(tmpdir: &TempDir) {
    let mut sync_cmd = Command::cargo_bin("pcli").unwrap();
    sync_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "sync"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sync_cmd.assert().success();
}

/// Look up a currently active validator on the testnet.
/// Will return the most bonded, which means the Penumbra Labs CI validator.
fn get_validator(tmpdir: &TempDir) -> String {
    let mut validator_cmd = Command::cargo_bin("pcli").unwrap();
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

#[ignore]
#[test]
fn transaction_send_from_addr_0_to_addr_1() {
    tracing_subscriber::fmt::try_init().ok();
    let tmpdir = load_wallet_into_tmpdir();

    // Create a memo that we can inspect later, to confirm transaction
    // is viewable post-send.
    let memo_text = "Time is an illusion. Lunchtime doubly so.";

    // Send to self: tokens were distributed to `ADDRESS_0_STR`, in our test
    // we'll send `TEST_ASSET` to `ADDRESS_1_STR` and then check our balance.
    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            ADDRESS_1_STR,
            "--memo",
            memo_text,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    // Look up the transaction id from the command output so we can view it,
    // to exercise the `pcli view tx` code.
    let send_stdout = send_cmd.unwrap().stdout;
    let tx_regex = Regex::new(r"transaction confirmed and detected: ([0-9a-f]{64})").unwrap();
    let s = std::str::from_utf8(&send_stdout).unwrap();
    let captures = tx_regex.captures(s);
    let tx_id = &captures
        .and_then(|x| x.get(1))
        .expect("can find transaction id within 'pcli send tx' output")
        .as_str();
    let mut view_cmd = Command::cargo_bin("pcli").unwrap();
    view_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "view",
            "tx",
            "--raw",
            tx_id,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    view_cmd.assert().success();

    // Convert the raw JSON to a protobuf TransactionView, then convert
    // that to a domain type.
    let view_output = view_cmd.output().unwrap();
    let view_stdout: String = std::str::from_utf8(&view_output.stdout)
        .unwrap()
        .to_string();
    let view_json: Value =
        serde_json::from_str(&view_stdout).expect("can parse JSON from 'pcli view tx'");

    let tvp: ProtoTransactionView = serde_json::value::from_value(view_json).unwrap();
    let tv: TransactionView = tvp.try_into().unwrap();

    assert!(matches!(
        &tv.body_view.action_views[0],
        penumbra_sdk_transaction::ActionView::Spend(_)
    ));

    // Inspect the TransactionView and ensure that we can read the memo text.
    let mv = tv
        .body_view
        .memo_view
        .expect("can find MemoView in TransactionView");
    match mv {
        penumbra_sdk_transaction::MemoView::Visible { plaintext, .. } => {
            tracing::info!(?plaintext, "plaintext memo");
            tracing::info!(?memo_text, "expected memo text");
            assert!(plaintext.text == memo_text);
        }
        penumbra_sdk_transaction::MemoView::Opaque { .. } => {
            panic!("MemoView for transaction was Opaque! We should be able to read this memo.");
        }
    }

    // Now we inspect our wallet balance to ensure the funds were transferred correctly.
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    // The 1 is the index of the address which should be separated from the
    // test_asset only by whitespace.
    balance_cmd
        .assert()
        .stdout(predicate::str::is_match(r"1\s*2019test_usd").unwrap());

    // Cleanup: Send the asset back at the end of the test such that other tests begin
    // from the original state.
    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "send",
            TEST_ASSET, // 1020test_usd
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
        .args(["--home", tmpdir.path().to_str().unwrap(), "tx", "sweep"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sweep_cmd.assert().success();
}

#[ignore]
#[test]
fn delegate_and_undelegate() {
    tracing_subscriber::fmt::try_init().ok();
    tracing::info!("delegate_and_undelegate");
    let tmpdir = load_wallet_into_tmpdir();

    // Get a validator from the testnet.
    let validator = get_validator(&tmpdir);

    // Now undelegate. We attempt `max_attempts` times in case an epoch boundary passes
    // while we prepare the delegation. See issues #1522, #2047.
    let max_attempts = 5;

    let mut num_attempts = 0;
    loop {
        tracing::info!(attempt_number = num_attempts, "attempting delegation");
        // Delegate a tiny bit of penumbra to the validator.
        let mut delegate_cmd = Command::cargo_bin("pcli").unwrap();
        delegate_cmd
            .args([
                "--home",
                tmpdir.path().to_str().unwrap(),
                "tx",
                "delegate",
                "1penumbra",
                "--to",
                validator.as_str(),
            ])
            .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
        let delegation_result = delegate_cmd.assert().try_success();
        tracing::debug!(?delegation_result, "delegation result");

        // If the undelegation command succeeded, we can exit this loop.
        if delegation_result.is_ok() {
            tracing::info!("delegation succeeded");
            break;
        } else {
            tracing::info!("delegation failed");
            num_attempts += 1;
            if num_attempts >= max_attempts {
                panic!("Exceeded max attempts for fallible command");
            }
        }
    }

    tracing::info!("check that we have some of the delegation token");
    // Check we have some of the delegation token for that validator now.
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
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

    tracing::info!("check passed, now undelegate");

    // Now undelegate. We attempt `max_attempts` times in case an epoch boundary passes
    // while we prepare the delegation. See issues #1522, #2047.
    let mut num_attempts = 0;
    loop {
        tracing::info!(attempt_number = num_attempts, "attempting undelegation");
        let mut undelegate_cmd = Command::cargo_bin("pcli").unwrap();
        undelegate_cmd
            .args([
                "--home",
                tmpdir.path().to_str().unwrap(),
                "tx",
                "undelegate",
                delegation_token_str,
            ])
            .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
        let undelegation_result = undelegate_cmd.assert().try_success();

        tracing::debug!("undelegation done");
        // If the undelegation command succeeded, we can exit this loop.
        if undelegation_result.is_ok() {
            break;
        } else {
            tracing::error!(?undelegation_result, "undelegation failed");
            num_attempts += 1;
            tracing::info!(num_attempts, max_attempts, "undelegation failed");
            if num_attempts >= max_attempts {
                panic!("Exceeded max attempts for fallible command");
            }
        }
    }

    tracing::info!("undelegation succeeded, wait an epoch before claiming.");

    // Wait for the epoch duration.
    thread::sleep(*UNBONDING_DELAY);
    tracing::info!("epoch passed, claiming now");
    let mut undelegate_claim_cmd = Command::cargo_bin("pcli").unwrap();
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
    sync(&tmpdir);
}

#[ignore]
#[test]
fn lp_management() {
    let tmpdir = load_wallet_into_tmpdir();

    // Create a liquidity position selling 1cube for 1penumbra each.
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@1gm",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);

    // Address 0 has an opened LPNFT.
    assert!(output.contains("1lpnft_opened"));

    // Get the asset id for the LPNFT so we can close it:
    let asset_id = output
        .split_whitespace()
        .find(|s| s.contains("1lpnft_opened"))
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .replace("1lpnft_opened_", "");

    // Close the LP.
    let mut close_cmd = Command::cargo_bin("pcli").unwrap();
    close_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "close",
            &asset_id,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);

    // Address 0 has a closed LPNFT.
    assert!(output.contains("1lpnft_closed"));

    // Get the asset id for the LPNFT so we can withdraw it:
    let asset_id = output
        .split_whitespace()
        .find(|s| s.contains("1lpnft_closed"))
        .unwrap()
        .split(' ')
        .next()
        .unwrap()
        .replace("1lpnft_closed_", "");

    // Withdraw the LP.
    let mut close_cmd = Command::cargo_bin("pcli").unwrap();
    close_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw",
            &asset_id,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();

    // Test close-all: first open a few LPs
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@1gm",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@1gm",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@1gm",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();

    // Validate there are three opened position NFTs
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);
    let opened = output.matches("lpnft_opened").count();
    assert_eq!(opened, 3);

    // Close all the opened positions
    let mut closeall_cmd = Command::cargo_bin("pcli").unwrap();
    closeall_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "close-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    closeall_cmd.assert().success();

    // Validate there are no longer any opened position NFTs
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);
    let opened = output.matches("lpnft_opened").count();
    assert_eq!(opened, 0);
    // Should be three closed positions
    let closed = output.matches("lpnft_closed").count();
    assert_eq!(closed, 3);

    // Withdraw all the closed positions
    let mut withdrawall_cmd = Command::cargo_bin("pcli").unwrap();
    withdrawall_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    withdrawall_cmd.assert().success();

    // Validate there are no longer any closed position NFTs
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);
    let closed = output.matches("lpnft_closed").count();
    assert_eq!(closed, 0);
}

#[ignore]
#[test]
/// Test that we can swap `gm` for `test_usd`
/// Setup:
/// There are two wallets, address 0 and address 1.
/// Address 0 has 100gm and 5001test_usd.
/// Address 1 has no gm and 1000test_usd.
/// Test:
/// Address 1 posts an order to sell 1test_usd for 1gm.
/// Address 0 swaps 1gm for 1test_usd.
/// Validate:
/// Address 0 has 99gm and 5002test_usd.
/// Address 1 has 1gm and 999test_usd.
fn swap() {
    let tmpdir = load_wallet_into_tmpdir();

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    balance_cmd
        .assert()
        // Address 0 has 100gm.
        .stdout(predicate::str::is_match(r"0\s*100gm").unwrap())
        // Address 1 has no gm.
        .stdout(
            predicate::str::is_match(r"1\s[0-9]*\.?[0-9]gm")
                .unwrap()
                .not(),
        )
        // Address 0 has some penumbra.
        .stdout(predicate::str::is_match(r"0\s*5001test_usd").unwrap())
        // Address 1 has 1000test_usd.
        .stdout(predicate::str::is_match(r"1\s*1000test_usd").unwrap());

    // Address 1: post an order to sell 1penumbra for 1gm.
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1test_usd@1gm",
            "--source",
            "1",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();

    balance_cmd
        .assert()
        // Address 1 still has no gm.
        .stdout(
            predicate::str::is_match(r"1\s[0-9]*\.?[0-9]gm")
                .unwrap()
                .not(),
        )
        // Address 1 has 999test_usd.
        .stdout(predicate::str::is_match(r"1\s*999test_usd").unwrap());

    // Address 1: swaps 1gm for 1penumbra.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "1gm",
            "--into",
            "test_usd",
            "--source",
            "0",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd.assert().success();

    // Sleep to allow the outputs from the swap to be processed.
    thread::sleep(*UNBONDING_DELAY);
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    balance_cmd
        .assert()
        // Address 0 has 99gm (swapped 1gm).
        .stdout(predicate::str::is_match(r"0\s*99gm").unwrap())
        // Address 0 has 5002test_usd
        .stdout(predicate::str::is_match(r"0\s*5002test_usd").unwrap())
        // Address 1 has no gm (needs to withdraw LP).
        .stdout(
            predicate::str::is_match(r"1\s[0-9]*\.?[0-9]gm")
                .unwrap()
                .not(),
        );

    // Close and withdraw any existing liquidity positions.
    let mut close_cmd = Command::cargo_bin("pcli").unwrap();
    close_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "close-all",
            "--source",
            "1",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();

    // Wait for processing.
    thread::sleep(*UNBONDING_DELAY);
    let mut withdraw_cmd = Command::cargo_bin("pcli").unwrap();
    withdraw_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw-all",
            "--source",
            "1",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    withdraw_cmd.assert().success();

    thread::sleep(*UNBONDING_DELAY);
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    balance_cmd
        .assert()
        // Address 0 has 99gm.
        .stdout(predicate::str::is_match(r"0\s*99gm").unwrap())
        // Address 0 has 5002test_usd
        .stdout(predicate::str::is_match(r"0\s*5002test_usd").unwrap())
        // Address 1 has 1gm.
        .stdout(predicate::str::is_match(r"1\s*1gm").unwrap())
        // Address 1 has 999test_usd
        .stdout(predicate::str::is_match(r"1\s*999test_usd").unwrap());
}

#[ignore]
#[test]
fn governance_submit_proposal() {
    let tmpdir = load_wallet_into_tmpdir();

    // Get template for signaling proposal.
    let mut template_cmd = Command::cargo_bin("pcli").unwrap();
    template_cmd
        .args([
            "--home",
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
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "submit",
            "--file",
            "proposal.toml",
            "--deposit-amount",
            "10penumbra",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    submit_cmd.assert().success();

    // Now list the proposals.
    let mut proposals_cmd = Command::cargo_bin("pcli").unwrap();
    proposals_cmd
        .args([
            "--home",
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
    let tmpdir = load_wallet_into_tmpdir();
    let validator = get_validator(&tmpdir);
    let mut query_cmd = Command::cargo_bin("pcli").unwrap();
    query_cmd
        .args([
            "--home",
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
            "--home",
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
            "--home",
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
            "--home",
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

    // Now we retrieve the actual cometbft consensus key from the network data dir.
    // Doing so assumes that the generated data was previously but in place,
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
        "network_data",
        "node0",
        "cometbft",
        "config",
        "priv_validator_key.json",
    ]
    .iter()
    .collect();
    let tm_key_config: Value =
        serde_json::from_str(&std::fs::read_to_string(tm_key_filepath).unwrap())
            .expect("Could not read cometbft key config file");
    let tm_key: tendermint::PublicKey =
        serde_json::value::from_value(tm_key_config["pub_key"].clone())
            .expect("Could not parse cometbft key config file");

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
            "--home",
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

#[ignore]
#[test]
fn test_orders() {
    let tmpdir = load_wallet_into_tmpdir();

    // Close and withdraw any existing liquidity positions.
    let mut close_cmd = Command::cargo_bin("pcli").unwrap();
    close_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "close-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();
    let mut withdraw_cmd = Command::cargo_bin("pcli").unwrap();
    withdraw_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    withdraw_cmd.assert().success();

    // Create a liquidity position selling 1penumbra for 225test_usd each.
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@225test_usd",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();

    // Swap 225test_usd for some penumbra. In theory, we should receive ~1 penumbra for 225test_usd
    // based on the position above. In practice, we'll receive slightly less due to rounding: 0.99999penumbra.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "225test_usd",
            "--into",
            "penumbra",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd
        .assert()
        .stdout(
            predicate::str::is_match(
                "You will receive outputs of 0test_usd and 999.999mpenumbra. Claiming now...",
            )
            .unwrap(),
        )
        .success();

    // The position should now have test_usd reserves, so we can swap against it again...

    // Swap 1penumbra for some test_usd. We expect to receive 225test_usd for 1penumbra
    // based on the position above.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "1penumbra",
            "--into",
            "test_usd",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd
        .assert()
        .stdout(
            predicate::str::is_match(
                "You will receive outputs of 225test_usd and 0penumbra. Claiming now...",
            )
            .unwrap(),
        )
        .success();

    // Close and withdraw any existing liquidity positions.
    let mut close_cmd = Command::cargo_bin("pcli").unwrap();
    close_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "close-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();
    let mut withdraw_cmd = Command::cargo_bin("pcli").unwrap();
    withdraw_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    withdraw_cmd.assert().success();

    // Create a liquidity position buying 1penumbra for 225test_usd each.
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "buy",
            "1penumbra@225test_usd",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();

    // Swap 1penumbra for some test_usd. We expect to receive 225test_usd for 1penumbra
    // based on the position above.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "1penumbra",
            "--into",
            "test_usd",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd
        .assert()
        .stdout(
            predicate::str::is_match(
                "You will receive outputs of 225test_usd and 0penumbra. Claiming now...",
            )
            .unwrap(),
        )
        .success();

    // The position should now have some penumbra reserves, so we can swap against it again...
    // Swap 225test_usd for some penumbra. We expect to receive 1penumbra for 225test_usd
    // based on the position above.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "225test_usd",
            "--into",
            "penumbra",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd
        .assert()
        .stdout(
            predicate::str::is_match(
                "You will receive outputs of 0test_usd and 999.999mpenumbra. Claiming now...",
            )
            .unwrap(),
        )
        .success();

    // Close and withdraw any existing liquidity positions.
    let mut close_cmd = Command::cargo_bin("pcli").unwrap();
    close_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "close-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();
    let mut withdraw_cmd = Command::cargo_bin("pcli").unwrap();
    withdraw_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw-all",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    withdraw_cmd.assert().success();
}

#[ignore]
#[test]
fn delegate_submit_proposal_and_vote() {
    let tmpdir = load_wallet_into_tmpdir();

    // Get a validator from the testnet.
    let validator = get_validator(&tmpdir);

    // Now undelegate. We attempt `max_attempts` times in case an epoch boundary passes
    // while we prepare the delegation. See issues #1522, #2047.
    let max_attempts = 5;

    let mut num_attempts = 0;
    loop {
        // Delegate a tiny bit of penumbra to the validator.
        let mut delegate_cmd = Command::cargo_bin("pcli").unwrap();
        delegate_cmd
            .args([
                "--home",
                tmpdir.path().to_str().unwrap(),
                "tx",
                "delegate",
                "1penumbra",
                "--to",
                validator.as_str(),
            ])
            .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
        let delegation_result = delegate_cmd.assert().try_success();

        // If the undelegation command succeeded, we can exit this loop.
        if delegation_result.is_ok() {
            break;
        } else {
            num_attempts += 1;
            if num_attempts >= max_attempts {
                panic!("Exceeded max attempts for fallible command");
            }
        }
    }

    // Check we have some of the delegation token for that validator now.
    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "balance"])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    balance_cmd
        .assert()
        .stdout(predicate::str::is_match(validator.as_str()).unwrap());

    let mut proposal_template_cmd = Command::cargo_bin("pcli").unwrap();
    let template = proposal_template_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "template",
            "signaling",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let template_str = String::from_utf8(template).unwrap();
    let template_file = load_string_to_file(template_str, &tmpdir);
    let template_path = template_file.path().to_str().unwrap();

    let mut submit_proposal_cmd = Command::cargo_bin("pcli").unwrap();
    submit_proposal_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "proposal",
            "submit",
            "--file",
            template_path,
            "--deposit-amount",
            "10penumbra",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    submit_proposal_cmd.assert().success();
    sync(&tmpdir);

    let mut proposals_cmd = Command::cargo_bin("pcli").unwrap();
    proposals_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "query",
            "governance",
            "list-proposals",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS))
        .assert()
        .success()
        .stdout(predicate::str::is_match("A short title").unwrap());

    let mut vote_cmd = Command::cargo_bin("pcli").unwrap();
    vote_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "vote",
            "yes",
            "--on",
            "0",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS))
        .assert()
        .success();
}

#[ignore]
#[test]
/// Poll the CommunityPool RPC and confirm it returns correct info.
/// Then make a deposit, and query again, confirming the deposit worked.
fn community_pool_() {
    let tmpdir = load_wallet_into_tmpdir();
    // The default devnet config doesn't contain any CommunityPool allocations,
    // so we expect the balance to be `0penumbra`.
    let initial_balance = String::from("0penumbra");
    // We can deposit any amount here; we'll check that the CommunityPool
    // balance has increased by precisely this amount.
    let deposit_amount = String::from("5penumbra");
    let mut balance_check_1 = Command::cargo_bin("pcli").unwrap();
    balance_check_1
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "query",
            "community-pool",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    balance_check_1
        .assert()
        .stdout(predicate::str::is_match(format!("^{initial_balance}")).unwrap());

    let mut deposit_cmd = Command::cargo_bin("pcli").unwrap();
    deposit_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "community-pool-deposit",
            &deposit_amount,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    deposit_cmd.assert().success();

    let mut balance_check_2 = Command::cargo_bin("pcli").unwrap();
    balance_check_2
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "query",
            "community-pool",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    balance_check_2
        .assert()
        .stdout(predicate::str::is_match(format!("^{deposit_amount}")).unwrap());
}

#[ignore]
#[test]
/// Ensure that the view service can successfully parse all historical
/// transactions submitted above.
fn view_tx_hashes() {
    let tmpdir = load_wallet_into_tmpdir();
    let mut view_cmd = Command::cargo_bin("pcli").unwrap();
    view_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "view",
            "list-tx-hashes",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    let _view_result = view_cmd
        .assert()
        .try_success()
        .expect("pcli command failed: 'view list-tx-hashes'");
}
