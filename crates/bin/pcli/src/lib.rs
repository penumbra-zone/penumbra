#![deny(clippy::unwrap_used)]
#![allow(clippy::clone_on_copy)]

use {
    crate::{command::*, config::PcliConfig},
    anyhow::Result,
    camino::Utf8PathBuf,
    directories::ProjectDirs,
    futures::StreamExt,
    penumbra_sdk_proto::{
        box_grpc_svc::BoxGrpcService, custody::v1::custody_service_client::CustodyServiceClient,
        view::v1::view_service_client::ViewServiceClient,
    },
    penumbra_sdk_view::ViewClient,
    std::path::PathBuf,
};

pub mod command;
pub mod config;
pub mod opt;
pub mod warning;

mod dex_utils;
mod network;
mod terminal;
mod transaction_view_ext;

const CONFIG_FILE_NAME: &str = "config.toml";
const VIEW_FILE_NAME: &str = "pcli-view.sqlite";

#[derive(Debug)]
pub struct App {
    /// view will be `None` when a command indicates that it can be run offline via
    /// `.offline()` and Some(_) otherwise. Assuming `.offline()` has been implemenented
    /// correctly, this can be unwrapped safely.
    pub view: Option<ViewServiceClient<BoxGrpcService>>,
    pub custody: CustodyServiceClient<BoxGrpcService>,
    pub governance_custody: CustodyServiceClient<BoxGrpcService>,
    pub config: PcliConfig,
    /// If present, save the transaction here instead of broadcasting it.
    pub save_transaction_here_instead: Option<PathBuf>,
}

impl App {
    pub fn view(&mut self) -> &mut impl ViewClient {
        self.view.as_mut().expect("view service initialized")
    }

    pub async fn sync(&mut self) -> Result<()> {
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

pub fn default_home() -> Utf8PathBuf {
    let path = ProjectDirs::from("zone", "penumbra", "pcli")
        .expect("Failed to get platform data dir")
        .data_dir()
        .to_path_buf();
    Utf8PathBuf::from_path_buf(path).expect("Platform default data dir was not UTF-8")
}
