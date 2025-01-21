#![cfg(feature = "integration-testnet")]
//! Integration tests for communicating with the public Penumbra testnet.
//! These tests are off by default, given that they contact remote services,
//! but are useful to verify functionality for e.g. HTTPS connectivity.
//!
//! These tests duplicate some of the logic in the `network_integration`
//! test suite. It'd be helpful to refactor to keep things DRY later on.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

use penumbra_sdk_keys::test_keys::SEED_PHRASE;

/// The URL for the public testnet pd endpoint.
const NODE_URL: &str = "https://testnet.plinfra.net";

// Raise the command timeout because the long-lived testnet will have more blocks to sync.
// Syncing ~1,000,000 blocks on a mostly-empty wallet should not take ~10 minutes!
// See GH#4970 for a report of a recent slowdown in pcli syncing operations.
const TESTNET_TIMEOUT_COMMAND_SECONDS: u64 = 900;

/// Run "pcli view sync" against the testnet endpoint.
///
/// Mostly this test confirms that we have adequate Penumbra proving key and TLS config.
#[test]
fn sync_wallet_on_public_testnet() {
    tracing_subscriber::fmt::try_init().ok();
    let tmpdir = load_wallet_into_tmpdir_for_testnet();
    sync(&tmpdir);
}

/// Import the wallet from seed phrase into a temporary directory.
///
/// Ignores the `PENUMBRA_PD_NODE_URL` env var, preferring an explicit
/// CLI flag instead.
fn load_wallet_into_tmpdir_for_testnet() -> TempDir {
    let tmpdir = tempdir().unwrap();
    let mut setup_cmd = Command::cargo_bin("pcli").unwrap();
    setup_cmd
        .args([
            "--home",
            tmpdir.path().to_str().unwrap(),
            "init",
            "--grpc-url",
            NODE_URL,
            "soft-kms",
            "import-phrase",
        ])
        .write_stdin(SEED_PHRASE)
        .timeout(std::time::Duration::from_secs(5));
    setup_cmd
        .assert()
        .stdout(predicate::str::contains("Writing generated config"));
    tmpdir
}

/// Sync the wallet.
pub fn sync(tmpdir: &TempDir) {
    let mut sync_cmd = Command::cargo_bin("pcli").unwrap();
    sync_cmd
        .args(["--home", tmpdir.path().to_str().unwrap(), "view", "sync"])
        .timeout(std::time::Duration::from_secs(
            TESTNET_TIMEOUT_COMMAND_SECONDS,
        ));
    sync_cmd.assert().success();
}
