use std::env;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use penumbra_crypto::keys::{SeedPhrase, SpendKey};
use penumbra_crypto::FullViewingKey;
use penumbra_custody::policy::{AuthPolicy, PreAuthorizationPolicy};
use penumbra_custody::soft_kms::{self, SoftKms};
use penumbra_proto::{
    client::v1alpha1::oblivious_query_service_client::ObliviousQueryServiceClient,
    client::v1alpha1::ChainParametersRequest,
    custody::v1alpha1::custody_protocol_service_server::CustodyProtocolServiceServer,
    view::v1alpha1::view_protocol_service_server::ViewProtocolServiceServer,
};
use penumbra_view::{Storage, ViewService};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use std::fs;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use tonic::transport::Server;
use url::Url;

mod proxy;
pub use proxy::{ObliviousQueryProxy, SpecificQueryProxy, TendermintProxyProxy};

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PclientdConfig {
    /// FVK for both view and custody modes
    #[serde_as(as = "DisplayFromStr")]
    pub fvk: FullViewingKey,
    /// Optional KMS config for custody mode
    pub kms_config: Option<soft_kms::Config>,
}

impl PclientdConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(&self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

#[derive(Debug, Parser)]
#[clap(
    name = "pclientd",
    about = "The Penumbra view daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
    /// The path used to store pclientd state and config files.
    #[clap(long, env = "PENUMBRA_PCLIENTD_HOME")]
    pub home: Utf8PathBuf,
    /// The URL of the gRPC endpoint for pd.
    #[clap(
        short,
        long,
        default_value = "http://testnet.penumbra.zone:8080",
        env = "PENUMBRA_NODE_PD_URL"
    )]
    pub node: Url,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Initialize pclientd with the provided full viewing key (and optional seed phrase in custody mode)
    Init {
        /// The full viewing key to initialize the view service with.
        full_viewing_key: String,
        // If true, initialize in custody mode with the seed phrase provided to stdin
        #[clap(short, long)]
        custody: bool,
    },
    /// Start the view service.
    Start {
        /// Bind the view service to this socket.
        #[clap(long, env = "PENUMBRA_PCLIENTD_BIND", default_value = "127.0.0.1:8081")]
        bind_addr: SocketAddr,
    },
}

impl Opt {
    fn config_path(&self) -> Utf8PathBuf {
        let mut path = self.home.clone();
        path.push("config.toml");
        path
    }

    fn sqlite_path(&self) -> Utf8PathBuf {
        let mut path = self.home.clone();
        path.push("pclientd-db.sqlite");
        path
    }

    async fn init_sqlite(&self, fvk: &FullViewingKey) -> Result<Storage> {
        // Initialize client and storage
        let mut client = ObliviousQueryServiceClient::connect(self.node.to_string()).await?;

        let params = client
            .chain_parameters(tonic::Request::new(ChainParametersRequest {
                chain_id: String::new(),
            }))
            .await?
            .into_inner()
            .try_into()?;

        fs::create_dir_all(&self.home)?;

        Storage::initialize(Some(self.sqlite_path()), fvk.clone(), params).await
    }

    async fn load_or_init_sqlite(&self, fvk: &FullViewingKey) -> Result<Storage> {
        if self.sqlite_path().exists() {
            Ok(Storage::load(self.sqlite_path()).await?)
        } else {
            self.init_sqlite(fvk).await
        }
    }

    pub async fn exec(self) -> Result<()> {
        let opt = self;
        match &opt.cmd {
            Command::Init {
                full_viewing_key,
                custody,
            } => {
                let fvk = full_viewing_key.parse()?;
                opt.init_sqlite(&fvk).await?;

                println!(
                    "Initializing storage and configuration at: {:?}",
                    fs::canonicalize(&opt.home)?
                );

                // Read seed phrase from std_in if custody = true

                let seed_phrase = match custody {
                    false => None,
                    true => {
                        println!("Enter your seed phrase to enable pclientd custody mode: ");

                        let stdin = io::stdin();
                        let line = stdin
                            .lock()
                            .lines()
                            .next()
                            .expect("There was no next line.")
                            .expect("The line could not be read.");

                        Some(line)
                    }
                };

                // Create config file

                let kms_config: Option<soft_kms::Config> = match seed_phrase {
                    Some(seed_phrase) => {
                        let spend_key = SpendKey::from_seed_phrase(
                            SeedPhrase::from_str(seed_phrase.as_str())?,
                            0,
                        );

                        let pak = ed25519_consensus::SigningKey::new(rand_core::OsRng);
                        let pvk = pak.verification_key();

                        let auth_policy = vec![
                            AuthPolicy::OnlyIbcRelay,
                            AuthPolicy::DestinationAllowList {
                                allowed_destination_addresses: vec![
                                    spend_key
                                        .incoming_viewing_key()
                                        .payment_address(Default::default())
                                        .0,
                                ],
                            },
                            AuthPolicy::PreAuthorization(PreAuthorizationPolicy::Ed25519 {
                                required_signatures: 1,
                                allowed_signers: vec![pvk],
                            }),
                        ];
                        Some(soft_kms::Config {
                            spend_key,
                            auth_policy,
                        })
                    }
                    None => None,
                };

                let client_config = PclientdConfig {
                    kms_config,
                    fvk: FullViewingKey::from_str(full_viewing_key.as_ref())?,
                };

                let encoded = toml::to_string_pretty(&client_config).unwrap();

                // Write config to directory

                let config_file_path = &mut opt.home.clone();
                config_file_path.push("config.toml");
                let mut config_file = File::create(&config_file_path)?;

                config_file.write_all(encoded.as_bytes())?;

                Ok(())
            }
            Command::Start { bind_addr } => {
                tracing::info!(?opt.home, ?bind_addr, ?opt.node, "starting pclientd");

                let config = PclientdConfig::load(opt.config_path())?;
                let storage = opt.load_or_init_sqlite(&config.fvk).await?;

                let proxy_channel = tonic::transport::Channel::from_shared(opt.node.to_string())
                    .expect("this is a valid address")
                    .connect()
                    .await?;

                let oblivious_query_proxy = ObliviousQueryProxy(proxy_channel.clone());
                let specific_query_proxy = SpecificQueryProxy(proxy_channel.clone());
                let tendermint_proxy_proxy = TendermintProxyProxy(proxy_channel.clone());

                let view_service =
                    ViewProtocolServiceServer::new(ViewService::new(storage, opt.node).await?);
                let custody_service = config.kms_config.as_ref().map(|kms_config| {
                    CustodyProtocolServiceServer::new(SoftKms::new(
                        kms_config.spend_key.clone().into(),
                    ))
                });

                let server = Server::builder()
                    .accept_http1(true)
                    .add_service(tonic_web::enable(view_service))
                    .add_optional_service(custody_service.map(|s| tonic_web::enable(s)))
                    .add_service(tonic_web::enable(oblivious_query_proxy))
                    .add_service(tonic_web::enable(specific_query_proxy))
                    .add_service(tonic_web::enable(tendermint_proxy_proxy))
                    .serve(bind_addr.clone());

                tokio::spawn(server).await??;

                Ok(())
            }
        }
    }
}
