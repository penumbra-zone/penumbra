use anyhow::Result;
use directories::ProjectDirs;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

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
                println!("{}", d);
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
    /// Tendermint version, if tendermint is found on PATH.
    tendermint_version: Option<String>,
    /// pd version, if pd is found on PATH.
    pd_version: Option<String>,
    /// pcli version; baked in at compile time, so will always be present.
    pcli_version: String,
    /// Platform and architecture info for current host.
    uname: Option<String>,
    /// Status of directory for storing view info locally.
    pcli_data_directory: Option<std::path::PathBuf>,
    /// Status of custody keyfile, containing key material for pcli.
    pcli_keyfile: Option<std::path::PathBuf>,
    /// Historical custody keyfiles, archived for safekeeping.
    pcli_keyfiles_archived: Vec<String>,
}

impl DebugInfo {
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        let dd = Self::get_pcli_data_directory(data_dir);
        Self {
            tendermint_version: Self::get_tendermint_version(),
            pd_version: Self::get_pd_version(),
            pcli_version: Self::get_pcli_version(),
            uname: Self::get_uname(),
            pcli_data_directory: dd.clone(),
            pcli_keyfile: Self::get_pcli_custody_file(dd),
            pcli_keyfiles_archived: Self::get_pcli_custody_files_archived(),
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
        env!("VERGEN_GIT_SEMVER").to_string()
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
    /// Check whether custody JSON file exists.
    fn get_pcli_custody_file(data_dir: Option<PathBuf>) -> Option<PathBuf> {
        match data_dir.clone() {
            Some(dd) => {
                let mut k = dd;
                k.push("custody.json");
                if k.exists() {
                    Some(k)
                } else {
                    None
                }
            }
            None => None,
        }
    }
    /// Check whether archived custody keyfiles are available on the system.
    fn get_pcli_custody_files_archived() -> Vec<String> {
        // Here we re-implement the path-building logic from
        // `pcli::command::keys::archive_wallet`.
        let archive_dir = ProjectDirs::from("zone", "penumbra", "penumbra-testnet-archive")
            .expect("can build archive directory path");
        let dd = archive_dir.data_dir();

        // Walk archive directory and collect all "custody.json" files.
        let mut archived_files = Vec::<String>::new();
        for entry in WalkDir::new(dd.to_str().unwrap()) {
            let entry = entry.unwrap();
            if let Some(f) = entry.path().file_name() {
                if f.to_str().unwrap_or("") == "custody.json" {
                    archived_files.push(format!("{}", entry.path().display()));
                }
            }
        }
        archived_files
    }
}
