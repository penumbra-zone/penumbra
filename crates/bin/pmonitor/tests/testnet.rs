#![cfg(feature = "integration-testnet")]
//! Integration tests for pmonitor against the public Penumbra testnet.
//! Mostly useful for verifying that HTTPS connections for gRPC
//! are well supported.
use anyhow::Result;
use assert_cmd::Command as AssertCommand;
use pcli::config::PcliConfig;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tempfile::{tempdir, NamedTempFile, TempDir};

const NODE_URL: &str = "https://testnet.plinfra.net";

/// Initialize a new pcli wallet at the target directory.
/// Discards the generated seed phrase.
fn pcli_init_softkms(pcli_home: &TempDir) -> Result<()> {
    let mut cmd = AssertCommand::cargo_bin("pcli")?;
    cmd.args([
        "--home",
        pcli_home
            .path()
            .to_str()
            .expect("can convert wallet path to string"),
        "init",
        "--grpc-url",
        NODE_URL,
        "soft-kms",
        "generate",
    ])
    // send empty string to accept the interstitial seed phrase display
    .write_stdin("");
    cmd.assert().success();
    Ok(())
}

/// Retrieve a FullViewingKey from a pcli home dir.
fn get_fvk_from_wallet_dir(pcli_home: &TempDir) -> Result<String> {
    let pcli_config_path = pcli_home.path().join("config.toml");
    let pcli_config = PcliConfig::load(
        pcli_config_path
            .to_str()
            .expect("failed to convert pcli wallet path to str"),
    )?;
    Ok(pcli_config.full_viewing_key.to_string())
}

/// Given a list of FVKs, formatted as Strings, write a JSON file
/// containing those FVKs, for use with pmonitor via the `--fvks` CLI flag.
fn write_fvks_json(fvks: Vec<String>, dest_filepath: &File) -> Result<()> {
    let mut w = BufWriter::new(dest_filepath);
    serde_json::to_writer(&mut w, &fvks)?;
    w.flush()?;
    Ok(())
}

#[test]
// Initialize an empty (i.e. random) wallet. We don't care about prior balances,
// because we're not exercising misbehavior: all we care about is that pmonitor
// can talk to an HTTPS endpoint and understand the blocks it pulls.
pub fn pmonitor_passes_with_empty_wallet_on_testnet() -> Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    let pcli_home = tempdir().unwrap();
    pcli_init_softkms(&pcli_home)?;

    let fvks = vec![get_fvk_from_wallet_dir(&pcli_home)?];
    let fvks_json = NamedTempFile::new()?;
    write_fvks_json(fvks, fvks_json.as_file())?;
    let pmonitor_pardir = tempfile::tempdir()?;
    let pmonitor_home = initialize_pmonitor(&pmonitor_pardir, fvks_json.path())?;

    // Run `pmonitor audit` based on the pcli wallets and associated FVKs.
    let cmd = AssertCommand::cargo_bin("pmonitor")?
        .args([
            "--home",
            pmonitor_home
                .as_path()
                .to_str()
                .expect("failed to parse pmonitor tempdir as directory"),
            "audit",
        ])
        .ok();

    if cmd.is_ok() {
        Ok(())
    } else {
        anyhow::bail!("failed during 'pmonitor audit'")
    }
}

/// Generate a config directory for `pmonitor`, based on input FVKs.
fn initialize_pmonitor(tmpdir: &TempDir, fvks_json: &Path) -> anyhow::Result<PathBuf> {
    // pmonitor doesn't like pre-existing homedirs so we'll nest this one.
    let pmonitor_home = tmpdir.path().join("pmonitor");

    let cmd = AssertCommand::cargo_bin("pmonitor")?
        .args([
            "--home",
            pmonitor_home
                .as_path()
                .to_str()
                .expect("failed to parse pmonitor tempdir as dir"),
            "init",
            "--grpc-url",
            NODE_URL,
            "--fvks",
            fvks_json
                .to_str()
                .expect("failed to parse fvk json tempfile as filepath"),
        ])
        .output();

    assert!(
        cmd.unwrap().status.success(),
        "failed to initialize pmonitor"
    );
    Ok(pmonitor_home)
}
