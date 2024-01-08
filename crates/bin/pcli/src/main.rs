#![deny(clippy::unwrap_used)]
#![allow(clippy::clone_on_copy)]

use std::fs;

use anyhow::{Context, Result};
use clap::Parser;
use futures::StreamExt;

use box_grpc_svc::BoxGrpcService;
use command::*;
use config::PcliConfig;
use opt::Opt;
use penumbra_proto::{
    custody::v1alpha1::custody_protocol_service_client::CustodyProtocolServiceClient,
    view::v1alpha1::view_protocol_service_client::ViewProtocolServiceClient,
};
use penumbra_view::ViewClient;

mod box_grpc_svc;
mod command;
mod config;
mod dex_utils;
mod network;
mod opt;
mod terminal;
mod warning;

const CONFIG_FILE_NAME: &str = "config.toml";
const VIEW_FILE_NAME: &str = "pcli-view.sqlite";

#[derive(Debug)]
pub struct App {
    /// view will be `None` when a command indicates that it can be run offline via
    /// `.offline()` and Some(_) otherwise. Assuming `.offline()` has been implemenented
    /// correctly, this can be unwrapped safely.
    pub view: Option<ViewProtocolServiceClient<BoxGrpcService>>,
    pub custody: CustodyProtocolServiceClient<BoxGrpcService>,
    pub config: PcliConfig,
}

impl App {
    pub fn view(&mut self) -> &mut impl ViewClient {
        self.view.as_mut().expect("view service initialized")
    }

    async fn sync(&mut self) -> Result<()> {
        let mut status_stream =
            ViewClient::status_stream(self.view.as_mut().expect("view service initialized"))
                .await?;

        // Pull out the first message from the stream, which has the current state, and use
        // it to set up a progress bar.
        let initial_status = status_stream
            .next()
            .await
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("view service did not report sync status"))?;

        eprintln!(
            "Scanning blocks from last sync height {} to latest height {}",
            initial_status.full_sync_height, initial_status.latest_known_block_height,
        );

        use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
        let progress_bar = ProgressBar::with_draw_target(
            initial_status.latest_known_block_height - initial_status.full_sync_height,
            ProgressDrawTarget::stdout(),
        )
        .with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:50.cyan/blue} {pos:>7}/{len:7} {per_sec} ETA: {eta}"),
        );
        progress_bar.set_position(0);

        while let Some(status) = status_stream.next().await.transpose()? {
            progress_bar.set_position(status.full_sync_height - initial_status.full_sync_height);
        }
        progress_bar.finish();

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Display a warning message to the user so they don't get upset when all their tokens are lost.
    if std::env::var("PCLI_UNLEASH_DANGER").is_err() {
        warning::display();
    }

    let mut opt = Opt::parse();

    // Initialize tracing here, rather than when converting into an `App`, so
    // that tracing is set up even for wallet commands that don't build the `App`.
    opt.init_tracing();

    //Ensure that the data_path exists, in case this is a cold start
    fs::create_dir_all(&opt.home)
        .with_context(|| format!("Failed to create home directory {}", opt.home))?;

    // The init command takes the home dir directly, since it may need to
    // create the client state, so handle it specially here so that we can have
    // common code for the other subcommands.
    if let Command::Init(init_cmd) = &opt.cmd {
        init_cmd.exec(opt.home.as_path()).await?;
        return Ok(());
    }

    // The view reset command takes the home dir directly, and should not be invoked when there's a
    // view service running.
    if let Command::View(ViewCmd::Reset(reset)) = &opt.cmd {
        reset.exec(opt.home.as_path())?;
        return Ok(());
    }
    // The debug command takes the home dir directly
    if let Command::Debug(debug_cmd) = &opt.cmd {
        let dd = opt.home.into_std_path_buf();
        debug_cmd.exec(dd)?;
        return Ok(());
    }

    let (mut app, cmd) = opt.into_app().await?;

    if !cmd.offline() {
        app.sync().await?;
    }

    // TODO: this is a mess, figure out the right way to bundle up the clients + fvk
    // make sure to be compatible with client for remote view service, with different
    // concrete type

    match &cmd {
        Command::Init(_) => unreachable!("init command already executed"),
        Command::Debug(_) => unreachable!("debug command already executed"),
        Command::Transaction(tx_cmd) => tx_cmd.exec(&mut app).await?,
        Command::View(view_cmd) => view_cmd.exec(&mut app).await?,
        Command::Validator(cmd) => cmd.exec(&mut app).await?,
        Command::Query(cmd) => cmd.exec(&mut app).await?,
        Command::Ceremony(cmd) => cmd.exec(&mut app).await?,
        Command::Threshold(cmd) => cmd.exec(&mut app).await?,
    }

    Ok(())
}
