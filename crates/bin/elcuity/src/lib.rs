use anyhow::Context;
use clap::{Parser, Subcommand};

mod clients;
mod lp;
mod planner;
mod swap;
mod vote;

/// A suite for automated tournament actions.
///
/// Relies on an external view and custody service.
#[derive(Debug, Parser)]
pub struct Opt {
    /// A URL for the gRPC endpoint of pd, for communicating with the network.
    #[clap(long)]
    grpc_url: String,
    /// A URL for the gRPC endpoint of the view service, e.g. pclientd.
    #[clap(long)]
    view_service: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Vote continuously for a given asset.
    ///
    /// This action will repeatedly vote in the liquidity tournament, for a given denom.
    /// The denom must be specified as the base denom of an IBC transfer asset, e.g. 'transfer/channel-1/uusdc'.
    /// In order to cast votes, you must have staked UM previously: the vote command will not
    /// delegate for you.
    Vote(vote::Opt),
    /// Provide liquidity.
    ///
    /// This action will create liquidity positions (LPs) and maintain them over time.
    Lp(lp::Opt),
    /// Swap between different assets.
    ///
    /// This action will repeatedly swap assets, executing trades on the DEX.
    /// It supports multi-hop trades by repetitions of the `--cycle` flag
    /// to declare denom to route through.
    Swap(swap::Opt),
}

impl Opt {
    /// Run the command with the parsed options
    pub async fn run(self) -> anyhow::Result<()> {
        let clients = clients::Clients::init(self.grpc_url, self.view_service)
            .await
            .context("failed to configure client connections")?;
        match self.command {
            Command::Vote(opt) => opt.run(&clients).await?,
            Command::Lp(opt) => opt.run(&clients).await?,
            Command::Swap(opt) => opt.run(&clients).await?,
        }
        Ok(())
    }
}
