use crate::{
    config::{CustodyConfig, GovernanceCustodyConfig, PcliConfig},
    terminal::ActualTerminal,
    App, Command,
};
use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use directories::ProjectDirs;
use penumbra_custody::soft_kms::SoftKms;
use penumbra_proto::box_grpc_svc;
use penumbra_proto::{
    custody::v1::{
        custody_service_client::CustodyServiceClient, custody_service_server::CustodyServiceServer,
    },
    view::v1::{view_service_client::ViewServiceClient, view_service_server::ViewServiceServer},
};
use penumbra_view::ViewServer;
use std::io::IsTerminal as _;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[clap(name = "pcli", about = "The Penumbra command-line interface.", version)]
pub struct Opt {
    #[clap(subcommand)]
    pub cmd: Command,
    /// The home directory used to store configuration and data.
    #[clap(long, default_value_t = default_home(), env = "PENUMBRA_PCLI_HOME")]
    pub home: Utf8PathBuf,
}

impl Opt {
    pub fn init_tracing(&mut self) {
        tracing_subscriber::fmt()
            .with_ansi(std::io::stdout().is_terminal())
            .with_env_filter(
                EnvFilter::from_default_env()
                    // Without explicitly disabling the `r1cs` target, the ZK proof implementations
                    // will spend an enormous amount of CPU and memory building useless tracing output.
                    .add_directive(
                        "r1cs=off"
                            .parse()
                            .expect("rics=off is a valid filter directive"),
                    ),
            )
            .with_writer(std::io::stderr)
            .init();
    }

    pub fn load_config(&self) -> Result<PcliConfig> {
        let path = self.home.join(crate::CONFIG_FILE_NAME);
        PcliConfig::load(path)
    }

    pub async fn into_app(self) -> Result<(App, Command)> {
        let config = self.load_config()?;

        // Build the custody service...
        let custody = match &config.custody {
            CustodyConfig::ViewOnly => {
                tracing::info!("using view-only custody service");
                let null_kms = penumbra_custody::null_kms::NullKms::default();
                let custody_svc = CustodyServiceServer::new(null_kms);
                CustodyServiceClient::new(box_grpc_svc::local(custody_svc))
            }
            CustodyConfig::SoftKms(config) => {
                tracing::info!("using software KMS custody service");
                let soft_kms = SoftKms::new(config.clone());
                let custody_svc = CustodyServiceServer::new(soft_kms);
                CustodyServiceClient::new(box_grpc_svc::local(custody_svc))
            }
            CustodyConfig::Threshold(config) => {
                tracing::info!("using manual threshold custody service");
                let threshold_kms =
                    penumbra_custody::threshold::Threshold::new(config.clone(), ActualTerminal);
                let custody_svc = CustodyServiceServer::new(threshold_kms);
                CustodyServiceClient::new(box_grpc_svc::local(custody_svc))
            }
            CustodyConfig::Encrypted(config) => {
                tracing::info!("using encrypted custody service");
                let encrypted_kms =
                    penumbra_custody::encrypted::Encrypted::new(config.clone(), ActualTerminal).await?;
                let custody_svc = CustodyServiceServer::new(encrypted_kms);
                CustodyServiceClient::new(box_grpc_svc::local(custody_svc))
            }
        };

        // Build the governance custody service...
        let governance_custody = match &config.governance_custody {
            Some(separate_governance_custody) => match separate_governance_custody {
                GovernanceCustodyConfig::SoftKms(config) => {
                    tracing::info!(
                        "using separate software KMS custody service for validator voting"
                    );
                    let soft_kms = SoftKms::new(config.clone());
                    let custody_svc = CustodyServiceServer::new(soft_kms);
                    CustodyServiceClient::new(box_grpc_svc::local(custody_svc))
                }
                GovernanceCustodyConfig::Threshold(config) => {
                    tracing::info!(
                        "using separate manual threshold custody service for validator voting"
                    );
                    let threshold_kms =
                        penumbra_custody::threshold::Threshold::new(config.clone(), ActualTerminal);
                    let custody_svc = CustodyServiceServer::new(threshold_kms);
                    CustodyServiceClient::new(box_grpc_svc::local(custody_svc))
                }
            },
            None => custody.clone(), // If no separate custody for validator voting, use the same one
        };

        // ...and the view service...
        let view = match (self.cmd.offline(), &config.view_url) {
            // In offline mode, don't construct a view service at all.
            (true, _) => None,
            (false, Some(view_url)) => {
                // Use a remote view service.
                tracing::info!(%view_url, "using remote view service");

                let ep = tonic::transport::Endpoint::new(view_url.to_string())?;
                Some(ViewServiceClient::new(box_grpc_svc::connect(ep).await?))
            }
            (false, None) => {
                // Use an in-memory view service.
                let path = self.home.join(crate::VIEW_FILE_NAME);
                tracing::info!(%path, "using local view service");

                let svc = ViewServer::load_or_initialize(
                    Some(path),
                    &config.full_viewing_key,
                    config.grpc_url.clone(),
                )
                .await?;

                // Now build the view and custody clients, doing gRPC with ourselves
                let svc = ViewServiceServer::new(svc);
                Some(ViewServiceClient::new(box_grpc_svc::local(svc)))
            }
        };

        let app = App {
            view,
            custody,
            governance_custody,
            config,
        };
        Ok((app, self.cmd))
    }
}

fn default_home() -> Utf8PathBuf {
    let path = ProjectDirs::from("zone", "penumbra", "pcli")
        .expect("Failed to get platform data dir")
        .data_dir()
        .to_path_buf();
    Utf8PathBuf::from_path_buf(path).expect("Platform default data dir was not UTF-8")
}
