// Rust analyzer complains without this (but rustc is happy regardless)
#![recursion_limit = "256"]
#![allow(clippy::clone_on_copy)]
use std::{net::SocketAddr, path::Path};

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use directories::ProjectDirs;
use futures::StreamExt;
use penumbra_crypto::{keys::SpendKey, FullViewingKey};
use penumbra_custody::{CustodyClient, SoftHSM};
use penumbra_proto::{
    custody::{
        custody_protocol_client::CustodyProtocolClient,
        custody_protocol_server::CustodyProtocolServer,
    },
    view::{view_protocol_client::ViewProtocolClient, view_protocol_server::ViewProtocolServer},
};
use penumbra_view::{ViewClient, ViewService};
use tracing_subscriber::EnvFilter;
use url::Url;

mod box_grpc_svc;
mod command;
mod legacy;
mod network;
mod wallet;
mod warning;

use wallet::Wallet;

const CUSTODY_FILE_NAME: &'static str = "custody.json";
const VIEW_FILE_NAME: &'static str = "pcli-view.sqlite";

use box_grpc_svc::BoxGrpcService;
use command::*;

#[derive(Debug, Parser)]
#[clap(
    name = "pcli",
    about = "The Penumbra command-line interface.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// The hostname of the pd+tendermint node.
    #[clap(
        short,
        long,
        default_value = "testnet.penumbra.zone",
        env = "PENUMBRA_NODE_HOSTNAME",
        parse(try_from_str = url::Host::parse)
    )]
    pub node: url::Host,
    /// The port to use to speak to tendermint's RPC server.
    #[clap(long, default_value_t = 26657, env = "PENUMBRA_TENDERMINT_PORT")]
    pub tendermint_port: u16,
    /// The port to use to speak to pd's gRPC server.
    #[clap(long, default_value_t = 8080, env = "PENUMBRA_PD_PORT")]
    pub pd_port: u16,
    #[clap(subcommand)]
    pub cmd: Command,
    /// The directory to store the wallet and view data in.
    #[clap(short, long, default_value_t = default_data_dir())]
    pub data_path: Utf8PathBuf,
    /// If set, use a remote view service instead of local synchronization.
    #[clap(short, long, env = "PENUMBRA_VIEW_ADDRESS")]
    pub view_address: Option<SocketAddr>,
    /// The filter for `pcli`'s log messages.
    #[clap( long, default_value_t = EnvFilter::new("warn"), env = "RUST_LOG")]
    pub trace_filter: EnvFilter,
}

#[derive(Debug)]
pub struct App {
    pub view: ViewProtocolClient<BoxGrpcService>,
    pub custody: CustodyProtocolClient<BoxGrpcService>,
    pub fvk: FullViewingKey,
    pub wallet: Wallet,
    pub pd_url: Url,
    pub tendermint_url: Url,
}

impl App {
    pub fn view(&mut self) -> &mut impl ViewClient {
        &mut self.view
    }

    async fn from_opts(opts: &Opt) -> Result<Self> {
        // Create the data directory if it is missing.
        std::fs::create_dir_all(&opts.data_path).context("Failed to create data directory")?;

        let custody_path = opts.data_path.join(CUSTODY_FILE_NAME);
        let legacy_wallet_path = opts.data_path.join(legacy::WALLET_FILE_NAME);

        // Try to auto-migrate the legacy wallet file to the new location, if:
        // - the legacy wallet file exists
        // - the new wallet file does not exist
        if legacy_wallet_path.exists() && !custody_path.exists() {
            legacy::migrate(&legacy_wallet_path, &custody_path.as_path())?;
        }

        // Build the custody service...
        let wallet = Wallet::load(custody_path)?;
        let soft_hsm = SoftHSM::new(vec![wallet.spend_key.clone()]);
        let custody_svc = CustodyProtocolServer::new(soft_hsm);
        let custody = CustodyProtocolClient::new(box_grpc_svc::local(custody_svc));

        let fvk = wallet.spend_key.full_viewing_key().clone();

        // ...and the view service...
        let view = opts.view_client(&fvk).await?;

        let mut tendermint_url = format!("http://{}", opts.node)
            .parse::<Url>()
            .with_context(|| format!("Invalid node URL: {}", opts.node))?;
        let mut pd_url = tendermint_url.clone();
        pd_url
            .set_port(Some(opts.pd_port))
            .expect("pd URL will not be `file://`");
        tendermint_url
            .set_port(Some(opts.tendermint_port))
            .expect("tendermint URL will not be `file://`");

        Ok(Self {
            view,
            custody,
            fvk,
            wallet,
            pd_url,
            tendermint_url,
        })
    }

    async fn sync(&mut self) -> Result<()> {
        let mut status_stream = ViewClient::status_stream(&mut self.view, self.fvk.hash()).await?;

        // Pull out the first message from the stream, which has the current state, and use
        // it to set up a progress bar.
        let initial_status = status_stream
            .next()
            .await
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("view service did not report sync status"))?;

        println!(
            "Scanning blocks from last sync height {} to latest height {}",
            initial_status.sync_height, initial_status.latest_known_block_height,
        );

        use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
        let progress_bar = ProgressBar::with_draw_target(
            initial_status.latest_known_block_height - initial_status.sync_height,
            ProgressDrawTarget::stdout(),
        )
        .with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:50.cyan/blue} {pos:>7}/{len:7} {per_sec} ETA: {eta}"),
        );
        progress_bar.set_position(0);

        while let Some(status) = status_stream.next().await.transpose()? {
            progress_bar.set_position(status.sync_height - initial_status.sync_height);
        }
        progress_bar.finish();

        Ok(())
    }
}

impl Opt {
    /// Constructs a [`ViewProtocolClient`] based on the command-line options.
    async fn view_client(
        &self,
        fvk: &FullViewingKey,
    ) -> Result<ViewProtocolClient<BoxGrpcService>> {
        let svc = if let Some(address) = self.view_address {
            // Use a remote view service.
            tracing::info!(%address, "using remote view service");

            let ep = tonic::transport::Endpoint::new(format!("http://{}", address))?;
            box_grpc_svc::connect(ep).await?
        } else {
            // Use an in-memory view service.
            let path = self.data_path.join(VIEW_FILE_NAME);
            tracing::info!(%path, "using local view service");

            let svc = ViewService::load_or_initialize(
                path,
                &fvk,
                self.node.clone(),
                self.pd_port,
                self.tendermint_port,
            )
            .await?;

            // Now build the view and custody clients, doing gRPC with ourselves
            let svc = ViewProtocolServer::new(svc);
            box_grpc_svc::local(svc)
        };

        Ok(ViewProtocolClient::new(svc))
    }
}

fn default_data_dir() -> Utf8PathBuf {
    let path = ProjectDirs::from("zone", "penumbra", "pcli")
        .expect("Failed to get platform data dir")
        .data_dir()
        .to_path_buf();
    Utf8PathBuf::from_path_buf(path).expect("Platform default data dir was not UTF-8")
}

#[tokio::main]
async fn main() -> Result<()> {
    // Display a warning message to the user so they don't get upset when all their tokens are lost.
    if std::env::var("PCLI_UNLEASH_DANGER").is_err() {
        warning::display();
    }

    let mut opt = Opt::parse();

    tracing_subscriber::fmt()
        .with_env_filter(std::mem::take(&mut opt.trace_filter))
        .init();

    // The wallet command takes the data dir directly, since it may need to
    // create the client state, so handle it specially here so that we can have
    // common code for the other subcommands.
    if let Command::Wallet(wallet_cmd) = &opt.cmd {
        wallet_cmd.exec(opt.data_path.as_path())?;
        return Ok(());
    }

    let mut app = App::from_opts(&opt).await?;

    if opt.cmd.needs_sync() {
        app.sync().await?;
    }

    // TODO: this is a mess, figure out the right way to bundle up the clients + fvk
    // make sure to be compatible with client for remote view service, with different
    // concrete type

    match &opt.cmd {
        Command::Wallet(_) => unreachable!("wallet command already executed"),
        Command::Sync => {
            // We have already synchronized the wallet above, so we can just return.
        }
        Command::Tx(tx_cmd) => {
            tx_cmd
                .exec(&opt, &app.fvk, &mut app.view, &mut app.custody)
                .await?
        }
        Command::Addr(addr_cmd) => addr_cmd.exec(&app.fvk)?,
        Command::Balance(balance_cmd) => balance_cmd.exec(&app.fvk, &mut app.view).await?,
        Command::Validator(cmd) => {
            cmd.exec(&opt, &app.wallet.spend_key, &mut app.view, &mut app.custody)
                .await?
        }
        Command::Stake(cmd) => {
            cmd.exec(&opt, &app.fvk, &mut app.view, &mut app.custody)
                .await?
        }
        Command::Chain(cmd) => cmd.exec(&opt, &app.fvk, &mut app.view).await?,
        Command::Q(cmd) => cmd.exec(&opt).await?,
    }

    Ok(())
}
