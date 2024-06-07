use clap::Parser;

/// This struct represents the command-line options
#[derive(Debug, Parser)]
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
}
