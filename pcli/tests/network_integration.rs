//! Basic integration testing of `pcli` versus a target testnet.
//!
//! These tests are marked with `#[ignore]`, but can be run with:
//! `cargo test --package pcli -- --ignored`
//!
//! Tests against `testnet-preview.penumbra.zone` by default, override with
//! environmental variable `PENUMBRA_NODE`.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributed 1cube.

use std::{thread, time};

use assert_cmd::{prelude::*, Command};
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

// This address is for test purposes, allocations were added beginning with
// the 016-Pandia testnet.
const TEST_SEED_PHRASE: &'static str = "benefit cherry cannon tooth exhibit law avocado spare tooth that amount pumpkin scene foil tape mobile shine apology add crouch situate sun business explain";

// These addresses both correspond to the test wallet above.
const TEST_ADDRESS_0: &'static str = "penumbrav1t19amxg2dsmv6kfgfu8w7qqeqc4kxhtagz6nk0vt4kvy4wc5r39hqs47z9qxq9g5cljje4zrnvxghzyn5a24mhxc93e6gy2qrmtl0hgcnelmy48stgmc2ryujm0xhfeqhmazxzft";
const TEST_ADDRESS_1: &'static str = "penumbrav1t1fgxxj6r6hq489hyn56dmh2aezq54c5gq56tnc7d8fm78j36frsmzpgcm0vy8yg56hdsu9a0ym3npmtvl8xwltknyy85q7ffq59759mnc9ww5z5xy2vpsuxazyxplx290uwment";

const TEST_ASSET: &'static str = "1cube";

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
            "wallet",
            "import-from-phrase",
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
    let server_host = option_env!("PENUMBRA_NODE").unwrap_or("testnet-preview.penumbra.zone");

    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "--node",
            server_host,
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            TEST_ADDRESS_1,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
    send_cmd.assert().stdout(predicate::str::contains(
        "transaction submitted successfully",
    ));

    // Wait for a couple blocks for the transaction to be confirmed.
    let block_time = time::Duration::from_secs(2 * BLOCK_TIME_SECONDS);
    thread::sleep(block_time);

    let mut balance_cmd = Command::cargo_bin("pcli").unwrap();
    balance_cmd
        .args(&[
            "--data-path",
            tmpdir.path().to_str().unwrap(),
            "--node",
            server_host,
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
            "--node",
            server_host,
            "tx",
            "send",
            TEST_ASSET,
            "--to",
            TEST_ADDRESS_0,
        ])
        .timeout(std::time::Duration::from_secs(TIMEOUT_COMMAND_SECONDS));
}
