//! Convenience methods for wrangling `pcli` CLI invocations,
//! via `cargo bin` commands, for use in integration testing.

use anyhow::{Context, Result};
use assert_cmd::Command as AssertCommand;
use penumbra_sdk_keys::{address::Address, FullViewingKey};
use std::path::PathBuf;
use std::str::FromStr;

/// Initialize a new pcli wallet at the target directory.
/// Discards the generated seed phrase.
pub fn pcli_init_softkms(pcli_home: &PathBuf) -> Result<()> {
    let mut cmd = AssertCommand::cargo_bin("pcli")?;
    cmd.args([
        "--home",
        pcli_home
            .to_str()
            .expect("can convert wallet path to string"),
        "init",
        "--grpc-url",
        "http://127.0.0.1:8080",
        "soft-kms",
        "generate",
    ])
    // send empty string to accept the interstitial seed phrase display
    .write_stdin("");
    cmd.assert().success();
    Ok(())
}

/// Convenience method for looking up `address 0` from
/// pcli wallet stored at `pcli_home`.
pub fn pcli_view_address(pcli_home: &PathBuf) -> Result<Address> {
    let output = AssertCommand::cargo_bin("pcli")?
        .args(["--home", pcli_home.to_str().unwrap(), "view", "address"])
        .output()
        .expect("failed to retrieve address from pcli wallet");

    // Convert output to String, to trim trailing newline.
    let mut a = String::from_utf8_lossy(&output.stdout).to_string();
    if a.ends_with('\n') {
        a.pop();
    }
    Address::from_str(&a).with_context(|| format!("failed to convert str to Address: '{}'", a))
}

/// Perform a `pcli migrate balance` transaction from the wallet at `pcli_home`,
/// transferring funds to the destination `FullViewingKey`.
pub fn pcli_migrate_balance(pcli_home: &PathBuf, fvk: &FullViewingKey) -> Result<()> {
    let mut cmd = AssertCommand::cargo_bin("pcli")?;
    cmd.args([
        "--home",
        pcli_home
            .to_str()
            .expect("can convert wallet path to string"),
        "migrate",
        "balance",
    ])
    // pipe FVK to stdin
    .write_stdin(fvk.to_string());
    cmd.assert().success();
    Ok(())
}
