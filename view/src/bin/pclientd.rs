#![allow(clippy::clone_on_copy)]
use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use penumbra_crypto::keys::{SeedPhrase, SpendKey};
use penumbra_crypto::FullViewingKey;
use penumbra_custody::policy::{AuthPolicy, PreAuthorizationPolicy};
use penumbra_custody::soft_kms::{self, SoftKms};
use penumbra_proto::client::v1alpha1::oblivious_query_service_client::ObliviousQueryServiceClient;
use penumbra_proto::client::v1alpha1::ChainParametersRequest;
use penumbra_proto::custody::v1alpha1::custody_protocol_service_server::CustodyProtocolServiceServer;
use penumbra_proto::view::v1alpha1::view_protocol_service_server::ViewProtocolServiceServer;
use penumbra_view::ViewService;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::{env, fs};
use tonic::transport::Server;
#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientConfig {
    /// Optional KMS config for custody mode
    pub kms_config: Option<soft_kms::Config>,
    /// FVK for both view and custody modes
    pub fvk: FullViewingKey,
}

#[derive(Debug, Parser)]
#[clap(
    name = "pclientd",
    about = "The Penumbra view daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    cmd: Command,
    /// The path used to store pclientd state and config files.
    #[clap(long)]
    home: Utf8PathBuf,
    /// The address of the pd+tendermint node.
    #[clap(short, long, default_value = "testnet.penumbra.zone")]
    node: String,
    /// The port to use to speak to pd's gRPC server.
    #[clap(long, default_value = "8080")]
    pd_port: u16,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
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
        /// Bind the view service to this host.
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the view gRPC server to this port.
        #[clap(long, default_value = "8081")]
        view_port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::parse();

    match opt.cmd {
        Command::Init {
            full_viewing_key,
            custody,
        } => {
            // Initialize client and storage

            let mut client = ObliviousQueryServiceClient::connect(format!(
                "http://{}:{}",
                opt.node, opt.pd_port
            ))
            .await?;

            let params = client
                .chain_parameters(tonic::Request::new(ChainParametersRequest {
                    chain_id: String::new(),
                }))
                .await?
                .into_inner()
                .try_into()?;

            fs::create_dir_all(&opt.home)?;

            let mut sqlite_path = opt.home.clone();
            sqlite_path.push("pclientd-db.sqlite");

            penumbra_view::Storage::initialize(
                <Utf8PathBuf as AsRef<Utf8Path>>::as_ref(&sqlite_path),
                FullViewingKey::from_str(full_viewing_key.as_ref())
                    .context("The provided string is not a valid FullViewingKey")?,
                params,
            )
            .await?;

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
                    let spend_key =
                        SpendKey::from_seed_phrase(SeedPhrase::from_str(seed_phrase.as_str())?, 0);

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

            let client_config = ClientConfig {
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

        Command::Start { host, view_port } => {
            tracing::info!(?opt.home, ?host, ?view_port, ?opt.node, ?opt.pd_port, "starting pclientd");

            let sqlite_path = &mut opt.home.clone();
            sqlite_path.push("pclientd-db.sqlite");

            println!(
                "Reading storage and configuration at: {:?}",
                fs::canonicalize(&opt.home)?
            );

            let storage = penumbra_view::Storage::load(<Utf8PathBuf as AsRef<Utf8Path>>::as_ref(
                &sqlite_path,
            ))
            .await?;

            let config_file_path = &mut opt.home.clone();
            config_file_path.push("config.toml");

            let config_contents = match fs::read_to_string(config_file_path) {
                // If successful return the files text as `contents`.
                // `c` is a local variable.
                Ok(c) => c,
                // Handle the `error` case.
                Err(_) => {
                    // Write `msg` to `stderr`.
                    eprintln!("Could not read file");
                    "".to_string()
                }
            };

            let config: ClientConfig = toml::from_str(&config_contents)?;

            println!(
                "Starting view service at node {:?} and port {:?}",
                &opt.node, &opt.pd_port
            );

            let service = ViewService::new(storage, opt.node, opt.pd_port).await?;

            match config.kms_config {
                None => {
                    // No key management config: start in view mode

                    println!("No spend key found in config, starting pclientd in View mode.");

                    tokio::spawn(
                        Server::builder()
                            .accept_http1(true)
                            .add_service(tonic_web::enable(ViewProtocolServiceServer::new(service)))
                            .serve(
                                format!("{host}:{view_port}")
                                    .parse()
                                    .expect("this is a valid address"),
                            ),
                    )
                    .await??;
                }
                Some(kms_config) => {
                    // Key management config & spend key present: start in custody mode

                    println!("Spend key found in config, starting pclientd in Custody mode.");

                    let spend_key = kms_config.spend_key;

                    let soft_kms = SoftKms::new(spend_key.clone().into());

                    let custody_svc = CustodyProtocolServiceServer::new(soft_kms);

                    tokio::spawn(
                        Server::builder()
                            .accept_http1(true)
                            .add_service(tonic_web::enable(ViewProtocolServiceServer::new(service)))
                            .add_service(tonic_web::enable(custody_svc))
                            .serve(
                                format!("{host}:{view_port}")
                                    .parse()
                                    .expect("this is a valid address"),
                            ),
                    )
                    .await??;
                }
            }

            Ok(())
        }
    }
}
