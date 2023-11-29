use anyhow::Result;

use crate::{terminal::ActualTerminal, App};

#[derive(Debug, clap::Subcommand)]
pub enum ThresholdCmd {
    /// Follow along with the threshold signing process
    Follow,
}

impl ThresholdCmd {
    pub fn offline(&self) -> bool {
        match self {
            ThresholdCmd::Follow => true,
        }
    }

    #[tracing::instrument(skip(self, app))]
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let config = match &app.config.custody {
            crate::config::CustodyConfig::Threshold(config) => config,
            _ => anyhow::bail!("this command can only be used with the threshold custody backend"),
        };
        penumbra_custody::threshold::follow(config, &ActualTerminal).await
    }
}
