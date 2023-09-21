use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use penumbra_keys::keys::FullViewingKey;
use penumbra_proto::view::v1alpha1::{
    view_protocol_service_client::ViewProtocolServiceClient,
    view_protocol_service_server::ViewProtocolServiceServer,
};
use penumbra_view::{ViewClient, ViewService};
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

#[derive(Debug, Parser)]
#[clap(
    name = "psumcoordd",
    about = "Penumbra summoning ceremony coordinator",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Start the coordinator.
    Start {
        #[clap(long, env = "PENUMBRA_PSUMCOORDD_HOME")]
        storage_path: Utf8PathBuf,
        #[clap(long)]
        full_viewing_key: FullViewingKey,
        #[clap(long)]
        node: Url,
    },
}

impl Opt {
    async fn exec(self) -> Result<()> {
        match self.cmd {
            Command::Start {
                storage_path,
                full_viewing_key,
                node,
            } => {
                println!("yeh");
                let view = ViewService::load_or_initialize(
                    Some(&storage_path),
                    &full_viewing_key,
                    node.clone(),
                )
                .await?;
                // Now build the view and custody clients, doing gRPC with ourselves
                let mut view = ViewProtocolServiceClient::new(ViewProtocolServiceServer::new(view));
                loop {
                    println!("hey: {:?}", ViewClient::assets(&mut view).await?);
                    sleep(Duration::from_secs(1)).await
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();

    opt.exec().await
}
