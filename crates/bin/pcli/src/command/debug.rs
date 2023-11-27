use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, clap::Subcommand)]
pub enum DebugCmd {
    /// Emit debugging info, useful for requesting support
    Info,
}

impl DebugCmd {
    pub fn offline(&self) -> bool {
        true
    }

    pub fn exec(&self, data_dir: PathBuf) -> Result<()> {
        match self {
            DebugCmd::Info => {
                let debug_info = DebugInfo::new(data_dir);
                // Using derived serialization as a cheap Display impl for formatting
                // the output. It's human-readable enough when pretty, plus we can parse it.
                let d = serde_json::to_string_pretty(&debug_info)?;
                println!("{d}");
                Ok(())
            }
        }
    }
}

/// Represents a fact sheet about system status, bottling up
/// common support-related questions like "What chain are you on?"
/// or "What version of Tendermint are you running?". Intended to display
/// output via `pcli debug info`, for ease of pasting into chat or issues.
// The DebugInfo struct is only used to print info to stdout,
// so its field names won't be accessed elsewhere; thus allow dead_code.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct DebugInfo {
    /// CometBFT version, if cometbft is found on PATH.
    cometbft_version: Option<String>,
    /// Tendermint version, if tendermint is found on PATH.
    // Preserved for a while during Tendermint -> CometBFT,
    // to aid in debugging.
    tendermint_version: Option<String>,
    /// pd version, if pd is found on PATH.
    pd_version: Option<String>,
    /// pcli version; baked in at compile time, so will always be present.
    pcli_version: String,
    /// Platform and architecture info for current host.
    uname: Option<String>,
    /// Status of directory for storing view info locally.
    pcli_data_directory: Option<std::path::PathBuf>,
    /// Status of pcli config TOML, containing key material for pcli.
    pcli_config_file: Option<std::path::PathBuf>,
}

impl DebugInfo {
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        let dd = Self::get_pcli_data_directory(data_dir);
        Self {
            cometbft_version: Self::get_cometbft_version(),
            tendermint_version: Self::get_tendermint_version(),
            pd_version: Self::get_pd_version(),
            pcli_version: Self::get_pcli_version(),
            uname: Self::get_uname(),
            pcli_data_directory: dd.clone(),
            pcli_config_file: Self::get_pcli_config_file(dd),
        }
    }
    /// Attempt to retrieve version info for Tendermint by running
    /// `tendermint version`. Depending on deployment, tendermint may not be on the PATH;
    /// it may be in container context that `pcli` doesn't have access to. That's OK:
    /// we'll just report `None` in that case.
    fn get_tendermint_version() -> Option<String> {
        let cmd = Command::new("tendermint").args(["version"]).output();
        match cmd {
            Ok(c) => match std::str::from_utf8(&c.stdout) {
                Ok(o) => Some(o.trim_end().to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
    /// Attempt to retrieve version info for CometBFT by running
    /// `cometbft version`. Depending on deployment, cometbft may not be on the PATH;
    /// it may be in container context that `pcli` doesn't have access to. That's OK:
    /// we'll just report `None` in that case.
    fn get_cometbft_version() -> Option<String> {
        let cmd = Command::new("cometbft").args(["version"]).output();
        match cmd {
            Ok(c) => match std::str::from_utf8(&c.stdout) {
                Ok(o) => Some(o.trim_end().to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
    /// Return host info, including kernel and architecture. Should work
    /// equally well on Linux or macOS; Windows will return None.
    fn get_uname() -> Option<String> {
        let cmd = Command::new("uname").args(["-a"]).output();
        match cmd {
            Ok(c) => match std::str::from_utf8(&c.stdout) {
                Ok(o) => Some(o.trim_end().to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }
    /// Return the version for `pcli` baked in at compile time.
    fn get_pcli_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
    /// Attempt to find `pd` on PATH, and return its version number. Depending on deployment,
    /// `pd` may not be on the path; it may in a container context elsewhere.
    fn get_pd_version() -> Option<String> {
        match Command::new("pd").args(["--version"]).output() {
            Ok(c) => match std::str::from_utf8(&c.stdout) {
                Ok(o) => Some(o.trim_end().to_string()),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }

    /// Check whether data dir, as provided by arg-parsing, exists.
    fn get_pcli_data_directory(data_dir: PathBuf) -> Option<PathBuf> {
        match data_dir.exists() {
            true => Some(data_dir),
            false => None,
        }
    }
    /// Check pcli config TOML file exists.
    fn get_pcli_config_file(data_dir: Option<PathBuf>) -> Option<PathBuf> {
        match data_dir {
            Some(dd) => {
                let mut k = dd;
                k.push("config.toml");
                if k.exists() {
                    Some(k)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
