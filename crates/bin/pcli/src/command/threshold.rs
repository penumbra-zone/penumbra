use anyhow::Result;
use penumbra_sdk_custody::threshold::Terminal;

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
        let config = match app.config.custody.clone() {
            CustodyConfig::Threshold(config) => Some(config),
            CustodyConfig::Encrypted(config) => {
                let password = ActualTerminal::default().get_password().await?;
                config.convert_to_threshold(&password)?
            }
            _ => None, // If not threshold, we can't sign using threshold config
        };
        let governance_config = match &app.config.governance_custody {
            Some(GovernanceCustodyConfig::Threshold(governance_config)) => {
                Some(governance_config.clone())
            }
            None => config.clone(), // If no governance config, use regular one
            _ => None,              // If not threshold, we can't sign using governance config
        };
        match self {
            ThresholdCmd::Sign => {
                penumbra_sdk_custody::threshold::follow(
                    config.as_ref(),
                    governance_config.as_ref(),
                    &ActualTerminal::default(),
                )
                .await
            }
        }
    }
}
