// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use directories::ProjectDirs;
use penumbra_custody::policy::{AuthPolicy, PreAuthorizationPolicy};
use penumbra_custody::soft_kms::{self, SoftKms};
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_keys::FullViewingKey;
use penumbra_proto::{
    core::app::v1alpha1::{
        query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
    },
    custody::v1alpha1::custody_service_server::CustodyServiceServer,
    view::v1alpha1::view_service_server::ViewServiceServer,
};
use penumbra_view::{Storage, ViewServer};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use std::fs;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use tonic::transport::Server;
use url::Url;

mod proxy;
pub use proxy::{
    AppQueryProxy, ChainQueryProxy, CompactBlockQueryProxy, DexQueryProxy, DexSimulationProxy,
    GovernanceQueryProxy, SctQueryProxy, ShieldedPoolQueryProxy, StakeQueryProxy,
    TendermintProxyProxy,
};

use crate::proxy::FeeQueryProxy;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PclientdConfig {
    /// FVK for both view and custody modes
    #[serde_as(as = "DisplayFromStr")]
    pub full_viewing_key: FullViewingKey,
    /// The URL of the gRPC endpoint used to talk to pd.
    pub grpc_url: Url,
    /// The address to bind to serve gRPC.
    pub bind_addr: SocketAddr,
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

fn default_home() -> Utf8PathBuf {
    let path = ProjectDirs::from("zone", "penumbra", "pclientd")
        .expect("Failed to get platform data dir")
        .data_dir()
        .to_path_buf();
    Utf8PathBuf::from_path_buf(path).expect("Platform default data dir was not UTF-8")
}

#[derive(Debug, Parser)]
#[clap(name = "pclientd", about = "The Penumbra view daemon.", version)]
pub struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
    /// The path used to store pclientd state and config files.
    #[clap(long, default_value_t = default_home(), env = "PENUMBRA_PCLIENTD_HOME")]
    pub home: Utf8PathBuf,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Generate configs for `pclientd` in view or custody mode.
    Init {
        /// If provided, initialize in view mode with the given full viewing key.
        #[clap(long, display_order = 100, value_name = "FULL_VIEWING_KEY")]
        view: Option<String>,
        /// If provided, initialize in custody mode with the given seed phrase.
        ///
        /// If the value '-' is provided, the seed phrase will be read from stdin.
        #[clap(long, display_order = 200)]
        custody: Option<String>,
        /// Sets the URL of the gRPC endpoint used to talk to pd.
        #[clap(
            long,
            display_order = 900,
            default_value = "https://grpc.testnet.penumbra.zone",
            parse(try_from_str = Url::parse)
        )]
        grpc_url: Url,
        /// Sets the address to bind to to serve gRPC.
        #[clap(long, display_order = 900, default_value = "127.0.0.1:8081")]
        bind_addr: SocketAddr,
    },
    /// Start running `pclientd`.
    Start {},
    /// Delete `pclientd` storage to reset local state.
    Reset {},
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

    fn check_home_nonempty(&self) -> Result<()> {
        if self.home.exists() {
            if !self.home.is_dir() {
                return Err(anyhow::anyhow!(
                    "The home directory {:?} is not a directory.",
                    self.home
                ));
            }
            let mut entries = fs::read_dir(&self.home)?.peekable();
            if entries.peek().is_some() {
                return Err(anyhow::anyhow!(
                    "The home directory {:?} is not empty, refusing to overwrite it",
                    self.home
                ));
            }
        } else {
            fs::create_dir_all(&self.home)?;
        }
        Ok(())
    }

    async fn init_sqlite(&self, fvk: &FullViewingKey, grpc_url: &Url) -> Result<Storage> {
        // Initialize client and storage
        let mut client = AppQueryServiceClient::connect(grpc_url.to_string()).await?;

        let params = client
            .app_parameters(tonic::Request::new(AppParametersRequest {}))
            .await?
            .into_inner()
            .try_into()?;

        Storage::initialize(Some(self.sqlite_path()), fvk.clone(), params).await
    }

    async fn load_or_init_sqlite(&self, fvk: &FullViewingKey, grpc_url: &Url) -> Result<Storage> {
        if self.sqlite_path().exists() {
            Ok(Storage::load(self.sqlite_path()).await?)
        } else {
            self.init_sqlite(fvk, grpc_url).await
        }
    }

    pub async fn exec(self) -> Result<()> {
        let opt = self;
        match &opt.cmd {
            Command::Reset {} => {
                if opt.sqlite_path().exists() {
                    fs::remove_file(opt.sqlite_path())?;
                    println!("Deleted local storage at: {:?}", opt.sqlite_path());
                } else {
                    println!("No local storage at: {:?} (have you started pclientd, so it would have data to store?)", opt.sqlite_path());
                }

                Ok(())
            }
            Command::Init {
                view,
                custody,
                grpc_url,
                bind_addr,
            } => {
                // Check that the home directory is empty.
                opt.check_home_nonempty()?;

                let seed_phrase = match custody {
                    None => None,
                    Some(seed_phrase) => {
                        // Read seed phrase from std_in if '-' is supplied
                        if seed_phrase == "-" {
                            println!("Enter your seed phrase to enable pclientd custody mode: ");

                            let stdin = io::stdin();
                            let line = stdin
                                .lock()
                                .lines()
                                .next()
                                .expect("There was no next line.")
                                .expect("The line could not be read.");

                            Some(line)
                        } else {
                            Some(seed_phrase.clone())
                        }
                    }
                };

                let (spend_key, full_viewing_key) = match (seed_phrase, view) {
                    (Some(seed_phrase), None) => {
                        let spend_key = SpendKey::from_seed_phrase_bip44(
                            SeedPhrase::from_str(seed_phrase.as_str())?,
                            &Bip44Path::new(0),
                        );
                        let full_viewing_key = spend_key.full_viewing_key().clone();
                        (Some(spend_key), full_viewing_key)
                    }
                    (None, Some(view)) => (None, view.parse()?),
                    (None, None) => {
                        return Err(anyhow::anyhow!(
                            "Must provide either a seed phrase or a full viewing key."
                        ))
                    }
                    (Some(_), Some(_)) => {
                        return Err(anyhow::anyhow!(
                            "Cannot provide both a seed phrase and a full viewing key."
                        ))
                    }
                };

                println!(
                    "Initializing configuration at: {:?}",
                    fs::canonicalize(&opt.home)?
                );

                // Create config file with example authorization policy.
                let kms_config: Option<soft_kms::Config> = spend_key.map(|spend_key| {
                    // It's important that we throw away the signing key here, so that
                    // by default the config is "cannot spend funds" without manual editing.
                    let pak = ed25519_consensus::SigningKey::new(rand_core::OsRng);
                    let pvk = pak.verification_key();

                    let auth_policy = vec![
                        AuthPolicy::DestinationAllowList {
                            allowed_destination_addresses: vec![
                                spend_key
                                    .incoming_viewing_key()
                                    .payment_address(Default::default())
                                    .0,
                            ],
                        },
                        AuthPolicy::OnlyIbcRelay,
                        AuthPolicy::PreAuthorization(PreAuthorizationPolicy::Ed25519 {
                            required_signatures: 1,
                            allowed_signers: vec![pvk],
                        }),
                    ];
                    soft_kms::Config {
                        spend_key,
                        auth_policy,
                    }
                });

                let client_config = PclientdConfig {
                    kms_config,
                    full_viewing_key,
                    grpc_url: grpc_url.clone(),
                    bind_addr: *bind_addr,
                };

                let encoded = toml::to_string_pretty(&client_config)
                    .expect("able to convert client config to toml string");

                // Write config to directory

                let config_file_path = &mut opt.home.clone();
                config_file_path.push("config.toml");
                let mut config_file = File::create(&config_file_path)?;

                config_file.write_all(encoded.as_bytes())?;

                Ok(())
            }
            Command::Start {} => {
                let config = PclientdConfig::load(opt.config_path()).context(
                    "Failed to load pclientd config file. Have you run `pclientd init` with a FVK?",
                )?;

                tracing::info!(?opt.home, ?config.bind_addr, %config.grpc_url, "starting pclientd");
                let storage = opt
                    .load_or_init_sqlite(&config.full_viewing_key, &config.grpc_url)
                    .await?;

                let proxy_channel =
                    tonic::transport::Channel::from_shared(config.grpc_url.to_string())
                        .expect("this is a valid address")
                        .connect()
                        .await?;

                let app_query_proxy = AppQueryProxy(proxy_channel.clone());
                let governance_query_proxy = GovernanceQueryProxy(proxy_channel.clone());
                let dex_query_proxy = DexQueryProxy(proxy_channel.clone());
                let dex_simulation_proxy = DexSimulationProxy(proxy_channel.clone());
                let sct_query_proxy = SctQueryProxy(proxy_channel.clone());
                let fee_query_proxy = FeeQueryProxy(proxy_channel.clone());
                let shielded_pool_query_proxy = ShieldedPoolQueryProxy(proxy_channel.clone());
                let chain_query_proxy = ChainQueryProxy(proxy_channel.clone());
                let stake_query_proxy = StakeQueryProxy(proxy_channel.clone());
                let compact_block_query_proxy = CompactBlockQueryProxy(proxy_channel.clone());
                let tendermint_proxy_proxy = TendermintProxyProxy(proxy_channel.clone());

                let view_service =
                    ViewServiceServer::new(ViewServer::new(storage, config.grpc_url).await?);
                let custody_service = config.kms_config.as_ref().map(|kms_config| {
                    CustodyServiceServer::new(SoftKms::new(kms_config.spend_key.clone().into()))
                });

                let server = Server::builder()
                    .accept_http1(true)
                    .add_service(tonic_web::enable(view_service))
                    .add_optional_service(custody_service.map(tonic_web::enable))
                    .add_service(tonic_web::enable(app_query_proxy))
                    .add_service(tonic_web::enable(governance_query_proxy))
                    .add_service(tonic_web::enable(dex_query_proxy))
                    .add_service(tonic_web::enable(dex_simulation_proxy))
                    .add_service(tonic_web::enable(sct_query_proxy))
                    .add_service(tonic_web::enable(fee_query_proxy))
                    .add_service(tonic_web::enable(shielded_pool_query_proxy))
                    .add_service(tonic_web::enable(chain_query_proxy))
                    .add_service(tonic_web::enable(stake_query_proxy))
                    .add_service(tonic_web::enable(compact_block_query_proxy))
                    .add_service(tonic_web::enable(tendermint_proxy_proxy))
                    .add_service(tonic_web::enable(
                        tonic_reflection::server::Builder::configure()
                            .register_encoded_file_descriptor_set(
                                penumbra_proto::FILE_DESCRIPTOR_SET,
                            )
                            .build()
                            .with_context(|| "could not configure grpc reflection service")?,
                    ))
                    .serve(config.bind_addr);

                tokio::spawn(server).await??;

                Ok(())
            }
        }
    }
}
