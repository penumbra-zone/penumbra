use clap::{Parser, Subcommand};

mod clients;
mod vote;

/// A suite for automated tournament actions.
///
/// Relies on an external view and custody service.
#[derive(Debug, Parser)]
pub struct Opt {
    /// A URL for the GRPC endpoint.
    #[clap(long)]
    grpc_url: String,
    /// A URL for the view service.
    #[clap(long)]
    view_service: String,
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
        let clients = clients::Clients::init(self.grpc_url, self.view_service).await?;
        match self.command {
            Command::Vote(opt) => opt.run(&clients).await?,
        }
        Ok(())
    }
}
