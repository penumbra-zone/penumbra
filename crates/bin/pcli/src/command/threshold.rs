use anyhow::Result;

use crate::{terminal::ActualTerminal, App};

#[derive(Debug, clap::Subcommand)]
pub enum ThresholdCmd {
    /// Contribute to signing a transaction with threshold custody
    Sign,
}

impl ThresholdCmd {
    pub fn offline(&self) -> bool {
        match self {
            ThresholdCmd::Sign => true,
        }
    }

    #[tracing::instrument(skip(self, app))]
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        let config = match &app.config.custody {
            crate::config::CustodyConfig::Threshold(config) => config,
            _ => anyhow::bail!("this command can only be used with the threshold custody backend"),
        };
        match self {
            ThresholdCmd::Sign => {
                penumbra_custody::threshold::follow(config, &ActualTerminal).await
            }
        }
    }
}
