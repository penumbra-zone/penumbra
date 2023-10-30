use crate::{
    box_grpc_svc,
    config::{CustodyConfig, PcliConfig},
    App, Command,
};
use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use directories::ProjectDirs;
use penumbra_custody::soft_kms::SoftKms;
use penumbra_proto::{
    custody::v1alpha1::{
        custody_protocol_service_client::CustodyProtocolServiceClient,
        custody_protocol_service_server::CustodyProtocolServiceServer,
    },
    view::v1alpha1::{
        view_protocol_service_client::ViewProtocolServiceClient,
        view_protocol_service_server::ViewProtocolServiceServer,
    },
};
use penumbra_view::ViewService;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[clap(
    name = "pcli",
    about = "The Penumbra command-line interface.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
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
                let custody_svc = CustodyProtocolServiceServer::new(null_kms);
                CustodyProtocolServiceClient::new(box_grpc_svc::local(custody_svc))
            }
            CustodyConfig::SoftKms(config) => {
                tracing::info!("using software KMS custody service");
                let soft_kms = SoftKms::new(config.clone());
                let custody_svc = CustodyProtocolServiceServer::new(soft_kms);
                CustodyProtocolServiceClient::new(box_grpc_svc::local(custody_svc))
            }
        };

        // ...and the view service...
        let view = match (self.cmd.offline(), &config.view_url) {
            // In offline mode, don't construct a view service at all.
            (true, _) => None,
            (false, Some(view_url)) => {
                // Use a remote view service.
                tracing::info!(%view_url, "using remote view service");

                let ep = tonic::transport::Endpoint::new(view_url.to_string())?;
                Some(ViewProtocolServiceClient::new(
                    box_grpc_svc::connect(ep).await?,
                ))
            }
            (false, None) => {
                // Use an in-memory view service.
                let path = self.home.join(crate::VIEW_FILE_NAME);
                tracing::info!(%path, "using local view service");

                let svc = ViewService::load_or_initialize(
                    Some(path),
                    &config.full_viewing_key,
                    config.grpc_url.clone(),
                )
                .await?;

                // Now build the view and custody clients, doing gRPC with ourselves
                let svc = ViewProtocolServiceServer::new(svc);
                Some(ViewProtocolServiceClient::new(box_grpc_svc::local(svc)))
            }
        };

        let app = App {
            view,
            custody,
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
