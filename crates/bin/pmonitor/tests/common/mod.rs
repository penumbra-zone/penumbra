//! Integration test helpers for `pmonitor`.
//! Contains logic to bootstrap a local devnet, complete with genesis
//! allocations for pre-existing wallets, so that `pmonitor` can audit
//! the behavior of those wallets on the target chain.

use anyhow::{Context, Result};
use assert_cmd::Command as AssertCommand;
use once_cell::sync::Lazy;
use pcli::config::PcliConfig;
use penumbra_sdk_keys::address::Address;
use process_compose_openapi_client::Client;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
pub mod pcli_helpers;
use crate::common::pcli_helpers::{pcli_init_softkms, pcli_view_address};

/// The TCP port for the process-compose API, used to start/stop devnet.
const PROCESS_COMPOSE_PORT: u16 = 8888;

/// The path in-repo to the `process-compose` manifest used for running a devnet,
/// relative to the current crate root. This is a minimal manifest, that only runs pd & cometbft.
static PROCESS_COMPOSE_MANIFEST_FILEPATH: Lazy<PathBuf> = Lazy::new(|| {
    let p: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "..",
        "..",
        "deployments",
        "compose",
        "process-compose.yml",
    ]
    .iter()
    .collect();
    p
});

/// The path to the root of the git repo, used for setting the working directory
/// when running `process-compose`.
static REPO_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    let p: PathBuf = [env!("CARGO_MANIFEST_DIR"), "../", "../", "../"]
        .iter()
        .collect();
    p
});

/// Manager for running suites of integration tests for `pmonitor`.
/// Only one instance should exist at a time! The test suites
/// assume access to global resources such as 8080/TCP for pd,
/// and a hardcoded directory in `/tmp/` for the pmonitor configs.
pub struct PmonitorTestRunner {
    /// Top-level directory for storing all integration test info,
    /// such as wallets and pd network state.
    pmonitor_integration_test_dir: PathBuf,
    /// How many client wallets to create for testing.
    num_wallets: u16,
}

/// Make sure to halt the running devnet, regardless of test pass/fail.
impl Drop for PmonitorTestRunner {
    fn drop(&mut self) {
        let _result = self.stop_devnet();
    }
}

impl PmonitorTestRunner {
    /// Create a new test runner environment.
    /// Caller must ensure no other instances exist, because this method
    /// will destroy existing test data directories.
    pub fn new() -> Self {
        // Ideally we'd use a tempdir but using a hardcoded dir for debugging.
        let p: PathBuf = ["/tmp", "pmonitor-integration-test"].iter().collect();
        // Nuke any pre-existing state
        if p.exists() {
            remove_dir_all(&p).expect("failed to remove directory for pmonitor integration tests");
        }
        // Ensure parent dir exists; other methods will create subdirs as necessary.
        create_dir_all(&p).expect("failed to create directory for pmonitor integration tests");
        Self {
            pmonitor_integration_test_dir: p,
            num_wallets: 10,
        }
    }
    // Return path for pmonitor home directory.
    // Does not create the path, because `pmonitor` will fail if its home already exists.
    pub fn pmonitor_home(&self) -> PathBuf {
        self.pmonitor_integration_test_dir.join("pmonitor")
    }
    // Create directory and return path for storing client wallets
    pub fn wallets_dir(&self) -> Result<PathBuf> {
        let p = self.pmonitor_integration_test_dir.join("wallets");
        create_dir_all(&p)?;
        Ok(p)
    }

    /// Initialize local pcli configs for all wallets specified in config.
    pub fn create_pcli_wallets(&self) -> anyhow::Result<()> {
        for i in 0..self.num_wallets - 1 {
            let pcli_home = self.wallets_dir()?.join(format!("wallet-{}", i));
            pcli_init_softkms(&pcli_home)?;
        }
        Ok(())
    }

    /// Iterate over all client wallets and return a `PcliConfig` for each.
    pub fn get_pcli_wallet_configs(&self) -> anyhow::Result<Vec<PcliConfig>> {
        let mut results = Vec::<PcliConfig>::new();
        for i in 0..self.num_wallets - 1 {
            let pcli_home = self.wallets_dir()?.join(format!("wallet-{}", i));
            let pcli_config_path = pcli_home.join("config.toml");
            let pcli_config = PcliConfig::load(
                pcli_config_path
                    .to_str()
                    .expect("failed to convert pcli wallet path to str"),
            )?;
            results.push(pcli_config);
        }
        Ok(results)
    }

    /// Iterate over all client wallets and return address 0 for each.
    pub fn get_pcli_wallet_addresses(&self) -> anyhow::Result<Vec<Address>> {
        let mut results = Vec::<Address>::new();
        for i in 0..self.num_wallets - 1 {
            let pcli_home = self.wallets_dir()?.join(format!("wallet-{}", i));
            let penumbra_address = pcli_view_address(&pcli_home)?;
            results.push(penumbra_address);
        }
        Ok(results)
    }
    /// Iterate over all client wallets, grab an FVK for each, write those
    /// FVKs to a local JSON file, and return the path to that file.
    pub fn get_pcli_wallet_fvks_filepath(&self) -> anyhow::Result<PathBuf> {
        let p = self.pmonitor_integration_test_dir.join("fvks.json");
        if !p.exists() {
            // We use a Vec<String> rather than Vec<FullViewingKey> so we get the string
            // representations
            let fvks: Vec<String> = self
                .get_pcli_wallet_configs()?
                .into_iter()
                .map(|c| c.full_viewing_key.to_string())
                .collect();
            let mut w = BufWriter::new(File::create(&p)?);
            serde_json::to_writer(&mut w, &fvks)?;
            w.flush()?;
        }
        Ok(p)
    }

    /// Create a CSV file of genesis allocations for all pcli test wallets.
    pub fn generate_genesis_allocations(&self) -> anyhow::Result<PathBuf> {
        let allocations_filepath = self.pmonitor_integration_test_dir.join("allocations.csv");

        // Generate file contents
        if !allocations_filepath.exists() {
            let mut w = BufWriter::new(File::create(&allocations_filepath)?);
            let csv_header = String::from("amount,denom,address\n");
            w.write(csv_header.as_bytes())?;
            for a in self.get_pcli_wallet_addresses()? {
                let allo = format!("1_000_000__000_000,upenumbra,{}\n1000,test_usd,{}\n", a, a);
                w.write(allo.as_bytes())?;
            }
            w.flush()?;
        }
        Ok(allocations_filepath)
    }

    /// Create a genesis event for the local devnet, with genesis allocations for all pcli wallets.
    /// This is a *destructive* action, as it removes the contents of the default pd network_data
    /// directory prior to generation.
    pub fn generate_network_data(&self) -> anyhow::Result<()> {
        // TODO: it'd be nice if we wrote all this network_data to a tempdir,
        // but instead we just reuse the default pd home.

        let reset_cmd = AssertCommand::cargo_bin("pd")?
            .args(["network", "unsafe-reset-all"])
            .output();
        assert!(
            reset_cmd.unwrap().status.success(),
            "failed to clear out prior local devnet config"
        );

        // Ideally we'd use a rust interface to compose the network config, rather than shelling
        // out to `pd`, but the current API for network config isn't ergonomic. Also, we get free
        // integration testing for the `pd` CLI by shelling out, which is nice.
        let cmd = AssertCommand::cargo_bin("pd")?
            .args([
                "network",
                "generate",
                "--chain-id",
                "penumbra-devnet-pmonitor",
                "--unbonding-delay",
                "50",
                "--epoch-duration",
                "50",
                "--proposal-voting-blocks",
                "50",
                "--timeout-commit",
                "3s",
                // we must opt in to fees, in order to test the migration functionality!
                "--gas-price-simple",
                "500",
                // include allocations for the generated pcli wallets
                "--allocations-input-file",
                &self
                    .generate_genesis_allocations()?
                    .to_str()
                    .expect("failed to convert allocations csv to str"),
            ])
            .output();
        assert!(
            cmd.unwrap().status.success(),
            "failed to generate local devnet config"
        );
        Ok(())
    }

    /// Generate a config directory for `pmonitor`, based on input FVKs.
    pub fn initialize_pmonitor(&self) -> anyhow::Result<()> {
        let cmd = AssertCommand::cargo_bin("pmonitor")?
            .args([
                "--home",
                self.pmonitor_home()
                    .to_str()
                    .expect("failed to convert pmonitor home to str"),
                "init",
                "--grpc-url",
                "http://127.0.0.1:8080",
                "--fvks",
                self.get_pcli_wallet_fvks_filepath()
                    .context("failed to get wallet fvks")?
                    .to_str()
                    .expect("failed to convert fvks json filepath to str"),
            ])
            .output();

        assert!(
            cmd.unwrap().status.success(),
            "failed to initialize pmonitor"
        );
        Ok(())
    }

    /// Run `pmonitor audit` based on the pcli wallets and associated FVKs.
    pub fn pmonitor_audit(&self) -> anyhow::Result<()> {
        let p = self.pmonitor_integration_test_dir.join("pmonitor");
        let cmd = AssertCommand::cargo_bin("pmonitor")?
            .args([
                "--home",
                p.to_str().expect("failed to convert pmonitor home to str"),
                "audit",
            ])
            .ok();
        if cmd.is_ok() {
            Ok(())
        } else {
            anyhow::bail!("failed during 'pmonitor audit'")
        }
    }

    /// Halt any pre-existing local devnet for these integration tests.
    /// We assume that the port `8888` is unique to the process-compose API for this test suite.
    fn stop_devnet(&self) -> anyhow::Result<()> {
        // Confirm that process-compose is installed, otherwise integration tests can't run.
        Command::new("process-compose")
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("process-compose is not available on PATH; activate the nix dev env");

        // Stop an existing devnet on the custom port; ignore error, since we don't know one is
        // running.
        let cmd = Command::new("process-compose")
            .env("PC_PORT_NUM", PROCESS_COMPOSE_PORT.to_string())
            .arg("down")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match cmd {
            Ok(_c) => {
                tracing::trace!(
                    "'process-compose down' completed, sleeping briefly during teardown"
                );

                std::thread::sleep(Duration::from_secs(2));
                return Ok(());
            }
            Err(_e) => {
                tracing::trace!(
                    "'process-compose down' failed, presumably no prior network running"
                );
                Ok(())
            }
        }
    }

    /// Run a local devnet based on input config. Returns a handle to the spawned process,
    /// so that cleanup can be handled gracefully.
    /// We assume that the port `8888` is unique to the process-compose API for this test suite.
    pub async fn start_devnet(&self) -> anyhow::Result<Child> {
        // Ensure no other instance is currently running;
        self.stop_devnet()?;

        self.generate_network_data()?;

        // Stop an existing devnet on the custom port; ignore error, since we don't know one is
        // running.
        let child = Command::new("process-compose")
            .env("PC_PORT_NUM", PROCESS_COMPOSE_PORT.to_string())
            .current_dir(REPO_ROOT.as_os_str())
            .args([
                "up",
                "--detached",
                "--config",
                PROCESS_COMPOSE_MANIFEST_FILEPATH
                    .to_str()
                    .expect("failed to convert process-compose manifest to str"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to execute devnet start cmd");

        // Use process-compose API to check for "Running" status on pd.
        let _pd_result = poll_for_ready("pd").await?;
        let _cmt_result = poll_for_ready("cometbft").await?;
        tracing::debug!("all processes ready, devnet is running");
        Ok(child)
    }
}

/// Block until the process-compose service denoted by `process_name` reports "Ready"
/// in its status API. Polls once per second, timing out after 60s.
async fn poll_for_ready(process_name: &str) -> anyhow::Result<()> {
    // Connect to the running process-compose service, via the custom port.
    let c = Client::new(format!("http://localhost:{}", PROCESS_COMPOSE_PORT).as_str());

    // Configure timeout, so we can error out if the service never comes up.
    let timeout = 60;
    let mut elapsed = 0;
    while elapsed < timeout {
        let resp = c.get_process(process_name).await;
        // Ignore error to API server, process-compose may not be up yet.
        if let Ok(r) = resp {
            let state = r.into_inner().is_ready;
            match state.as_deref() {
                Some("-") => {
                    tracing::debug!("still waiting for process to be ready: {}", process_name);
                }
                Some("Ready") => {
                    tracing::debug!("process '{}' is ready!", process_name);
                    return Ok(());
                }
                _ => {
                    tracing::warn!(
                        "unexpected status for process '{}', waiting...",
                        process_name
                    );
                }
            }
        }
        // Sleep and try again
        tokio::time::sleep(Duration::from_secs(1)).await;
        elapsed = elapsed + 1;
    }
    anyhow::bail!(
        "process '{}' not ready after {} seconds, failing",
        process_name,
        timeout
    );
}
