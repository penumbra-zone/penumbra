#![cfg(feature = "integration-testnet")]
//! Integration tests for communicating with the public Penumbra testnet.
//! These tests are off by default, given that they contact remote services,
//! but are useful to verify functionality for e.g. HTTPS connectivity.
use assert_cmd::Command;

/// The URL for the public testnet pd endpoint.
const NODE_URL: &str = "https://testnet.plinfra.net";

// Raise the command timeout because the long-lived testnet will have more blocks to sync.
// Syncing ~1,000,000 blocks on a mostly-empty wallet should not take ~10 minutes!
// See GH#4970 for a report of a recent slowdown in pcli syncing operations.
const TESTNET_TIMEOUT_COMMAND_SECONDS: u64 = 900;

#[test]
fn stream_blocks_on_testnet() {
    let mut cmd = Command::cargo_bin("measure").unwrap();
    cmd.args(["--node", NODE_URL, "stream-blocks"])
        .timeout(std::time::Duration::from_secs(
            TESTNET_TIMEOUT_COMMAND_SECONDS,
        ));
    cmd.assert().success();
}
