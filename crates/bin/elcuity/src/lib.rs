use clap::{Parser, Subcommand};

mod vote;

/// A command line tool for Penumbra
#[derive(Debug, Parser)]
pub struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Vote continuously for a given asset.
    Vote(vote::Opt),
}

impl Opt {
    /// Run the command with the parsed options
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Vote(opt) => opt.run().await?,
        }
        Ok(())
    }
}
