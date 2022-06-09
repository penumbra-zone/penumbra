//! Basic integration testing of `pcli` versus a target testnet.
//!
//! These tests are marked with `#[ignore]`, but can be run with:
//! `cargo test --package pcli -- --ignored`
//!
//! Tests against `testnet-preview.penumbra.zone` by default, override with
//! environmental variable `PENUMBRA_NODE`.

use assert_cmd::{prelude::*, Command};
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

// This address is for test purposes, allocations were added beginning with
// the 016-Pandia testnet.
const TEST_SEED_PHRASE: &'static str = "benefit cherry cannon tooth exhibit law avocado spare tooth that amount pumpkin scene foil tape mobile shine apology add crouch situate sun business explain";

// These addresses both correspond to the test wallet above.
const TEST_ADDRESS_0: &'static str = "penumbrav1t19amxg2dsmv6kfgfu8w7qqeqc4kxhtagz6nk0vt4kvy4wc5r39hqs47z9qxq9g5cljje4zrnvxghzyn5a24mhxc93e6gy2qrmtl0hgcnelmy48stgmc2ryujm0xhfeqhmazxzft";
const TEST_ADDRESS_1: &'static str = "penumbrav1t1fgxxj6r6hq489hyn56dmh2aezq54c5gq56tnc7d8fm78j36frsmzpgcm0vy8yg56hdsu9a0ym3npmtvl8xwltknyy85q7ffq59759mnc9ww5z5xy2vpsuxazyxplx290uwment";

/// Import the wallet from seed phrase into a temporary directory.
fn load_wallet_into_tmpdir() -> TempDir {
    let tmpdir = tempdir().unwrap();

    let mut setup_cmd = Command::cargo_bin("pcli").unwrap();
    setup_cmd.args(&[
        "--data-path",
        tmpdir.path().to_str().unwrap(),
        "wallet",
        "import-from-phrase",
        TEST_SEED_PHRASE,
    ]);
    setup_cmd
        .assert()
        .stdout(predicate::str::contains("Saving backup wallet"));

    tmpdir
}

#[ignore]
#[test]
fn transaction_send_happy_path() {
    let tmpdir = load_wallet_into_tmpdir();

    // Send to self: tokens were distributed to `TEST_ADDRESS_0`, in our test
    // we'll send to `TEST_ADDRESS_1` and then check our balance.
    let server_host = option_env!("PENUMBRA_NODE").unwrap_or("testnet-preview.penumbra.zone");

    let mut send_cmd = Command::cargo_bin("pcli").unwrap();
    send_cmd.args(&[
        "--data-path",
        tmpdir.path().to_str().unwrap(),
        "--node",
        server_host,
        "tx",
        "send",
        "100penumbra",
        "--to",
        TEST_ADDRESS_1,
    ]);
    send_cmd.assert().stdout(predicate::str::contains("200 ok"));

    let balance_cmd = Command::cargo_bin("pcli").unwrap().args(&[
        "--data-path",
        tmpdir.path().to_str().unwrap(),
        "--node",
        server_host,
        "balance",
    ]);

    // TODO: Confirm balance updated locally.
}
