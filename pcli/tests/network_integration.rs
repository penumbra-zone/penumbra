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
use penumbra_app::stake::validator::ValidatorToml;
use predicates::prelude::*;
use regex::Regex;
use serde_json::Value;
use tempfile::{tempdir, NamedTempFile, TempDir};

use penumbra_chain::test_keys::{ADDRESS_0_STR, ADDRESS_1_STR, SEED_PHRASE};

// The number "20" is chosen so that this is bigger than u64::MAX
// when accounting for the 10e18 scaling factor from the base denom.
const TEST_ASSET: &str = "20test_usd";

// The maximum amount of time any command is allowed to take before we error.
const TIMEOUT_COMMAND_SECONDS: u64 = 20;

// The time to wait before attempting to perform an undelegation claim.
// By default the epoch duration is 100 blocks, the block time is ~500 ms,
// and the number of unbonding epochs is 2.
const UNBONDING_DURATION: Lazy<Duration> = Lazy::new(|| {
    let blocks: f64 = std::env::var("EPOCH_DURATION")
        .unwrap_or("100".to_string())
        .parse()
        .unwrap();
    Duration::from_secs((1.5 * blocks) as u64)
});

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

/// Sync the wallet.
fn sync(tmpdir: &TempDir) {
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

/// Look up a currently active validator on the testnet.
/// Will return the most bonded, which means the Penumbra Labs CI validator.
fn get_validator(tmpdir: &TempDir) -> String {
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

    // Send to self: tokens were distributed to `ADDRESS_0_STR`, in our test
    // we'll send `TEST_ASSET` to `ADDRESS_1_STR` and then check our balance.
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
                "--data-path",
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

    // Now undelegate. We attempt `max_attempts` times in case an epoch boundary passes
    // while we prepare the delegation. See issues #1522, #2047.
    let mut num_attempts = 0;
    loop {
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
        } else {
            num_attempts += 1;
            if num_attempts >= max_attempts {
                panic!("Exceeded max attempts for fallible command");
            }
        }
    }

    // Wait for the epoch duration.
    thread::sleep(*UNBONDING_DURATION);
    let mut undelegate_claim_cmd = Command::cargo_bin("pcli").unwrap();
    undelegate_claim_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "undelegate-claim",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    undelegate_claim_cmd.assert().success();
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
            "--data-path",
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
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
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
            "--data-path",
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
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
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
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "withdraw",
            &asset_id,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    close_cmd.assert().success();

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);

    // Address 0 has a withdrawn LPNFT.
    assert!(output.contains("1lpnft_withdrawn"));

    // Test close-all: first open a few LPs
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@1gm",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();
    sell_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1penumbra@1gm",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();
    sell_cmd
        .args([
            "--data-path",
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
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
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
            "--data-path",
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
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
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
            "--data-path",
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
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "view",
            "balance",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));

    let o = balance_cmd
        .output()
        .expect("unable to fetch balance")
        .stdout;
    let output = String::from_utf8_lossy(&o);
    let closed = output.matches("lpnft_closed").count();
    assert_eq!(closed, 0);
    // Should be three withdrawn positions
    let withdrawn = output.matches("lpnft_withdrawn").count();
    assert_eq!(withdrawn, 3);
}

#[ignore]
#[test]
fn swap() {
    let tmpdir = load_wallet_into_tmpdir();

    // Create a liquidity position selling 1cube for 1penumbra each.
    let mut sell_cmd = Command::cargo_bin("pcli").unwrap();
    sell_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "position",
            "order",
            "sell",
            "1cube@1penumbra",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    sell_cmd.assert().success();

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
        // Address 0 has no `cube`.
        .stdout(
            predicate::str::is_match(format!(r"0\s*[0-9]+.*cube"))
                .unwrap()
                .not(),
        )
        // Address 1 should also have no cube.
        .stdout(
            predicate::str::is_match(format!(r"1\s*[0-9]+.*cube"))
                .unwrap()
                .not(),
        )
        // Address 1 has 1001penumbra.
        .stdout(predicate::str::is_match(format!(r"1\s*1001penumbra")).unwrap())
        // Address 0 should have some penumbra
        .stdout(predicate::str::is_match(format!(r"0\s*[0-9]+.*penumbra")).unwrap());

    // Swap 1penumbra for some cube from address 1.
    let mut swap_cmd = Command::cargo_bin("pcli").unwrap();
    swap_cmd
        .args([
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "tx",
            "swap",
            "1penumbra",
            "--into",
            "cube",
            "--source",
            "1",
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    swap_cmd.assert().success();

    // Sleep to allow the outputs from the swap to be processed.
    thread::sleep(*UNBONDING_DURATION);
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
        // Address 1 has 1cube now
        .stdout(predicate::str::is_match(format!(r"1\s*1cube")).unwrap())
        // and address 0 has no cube.
        .stdout(
            predicate::str::is_match(format!(r"0\s*[0-9]+.*cube"))
                .unwrap()
                .not(),
        )
        // Address 1 spent 1penumbra.
        .stdout(predicate::str::is_match(format!(r"1\s*1000penumbra")).unwrap());
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
    let tmpdir = load_wallet_into_tmpdir();
    let validator = get_validator(&tmpdir);
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
