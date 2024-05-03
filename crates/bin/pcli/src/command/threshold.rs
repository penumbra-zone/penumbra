use anyhow::Result;

use crate::{
    config::{CustodyConfig, GovernanceCustodyConfig},
    terminal::ActualTerminal,
    App,
};

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
            CustodyConfig::Threshold(config) => Some(config),
            _ => None, // If not threshold, we can't sign using threshold config
        };
        let governance_config = match &app.config.governance_custody {
            Some(GovernanceCustodyConfig::Threshold(governance_config)) => Some(governance_config),
            None => config, // If no governance config, use regular one
            _ => None,      // If not threshold, we can't sign using governance config
        };
        match self {
            ThresholdCmd::Sign => {
                penumbra_custody::threshold::follow(config, governance_config, &ActualTerminal)
                    .await
            }
        }
    }
}
