#![deny(clippy::unwrap_used)]
#![allow(clippy::clone_on_copy)]

use {
    crate::{command::*, config::PcliConfig},
    anyhow::Result,
    futures::StreamExt,
    penumbra_proto::{
        box_grpc_svc::BoxGrpcService, custody::v1::custody_service_client::CustodyServiceClient,
        view::v1::view_service_client::ViewServiceClient,
    },
    penumbra_view::ViewClient,
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
