use std::{path::PathBuf, time::Duration};

use anyhow::{Error, Result};
use clap::Parser;

/// This struct represents the command-line options
#[derive(Clone, Debug, Parser)]
#[clap(
    name = "cometindex",
    about = "processes raw events emitted by cometbft applications",
    version
)]
pub struct Options {
    /// PostgreSQL database connection string for the source database with raw events
    #[clap(short, long)]
    pub src_database_url: String,

    /// PostgreSQL database connection string for the destination database with compiled data
    #[clap(short, long)]
    pub dst_database_url: String,

    /// Filter for only events with this chain ID.
    #[clap(short, long)]
    pub chain_id: Option<String>,

    /// The rate at which to poll for changes, in milliseconds.
    #[clap(short, long, default_value = "500", value_parser = parse_poll_ms)]
    pub poll_ms: Duration,

    /// A file path for the genesis file to use when initializing the indexer.
    #[clap(short, long)]
    pub genesis_json: PathBuf,

    /// By default, the program will run as a daemon, continuously polling the src database
    /// for new events. If --exit-on-catchup is set, the program will instead exit after
    /// it has indexed all events in the src database. Useful for batch jobs.
    #[clap(long)]
    pub exit_on_catchup: bool,
}

/// Parses a string containing a [`Duration`], represented as a number of milliseconds.
fn parse_poll_ms(s: &str) -> Result<Duration> {
    s.parse().map(Duration::from_millis).map_err(Error::from)
}
