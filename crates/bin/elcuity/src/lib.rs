use clap::{Parser, Subcommand};
use penumbra_sdk_proto::{box_grpc_svc, view::v1::view_service_client::ViewServiceClient};
use penumbra_sdk_view::ViewClient;

mod vote;

async fn create_view_client(url: String) -> anyhow::Result<Box<dyn ViewClient>> {
    let endpoint = tonic::transport::Endpoint::new(url)?;
    Ok(Box::new(ViewServiceClient::new(
        box_grpc_svc::connect(endpoint).await?,
    )))
}

/// A suite for automated tournament actions.
///
/// Relies on an external view and custody service.
#[derive(Debug, Parser)]
pub struct Opt {
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
        let mut view_client = create_view_client(self.view_service).await?;
        match self.command {
            Command::Vote(opt) => opt.run(view_client.as_mut()).await?,
        }
        Ok(())
    }
}
