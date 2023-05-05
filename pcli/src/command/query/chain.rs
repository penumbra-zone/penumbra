use anyhow::Result;
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_app::stake::validator;
use penumbra_proto::client::v1alpha1::{ChainParametersRequest, InfoRequest};

// TODO: remove this subcommand and merge into `pcli q`

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum ChainCmd {
    /// Display chain parameters.
    Params,
    /// Display information about the current chain state.
    Info {
        /// If true, will also display chain parameters.
        #[clap(short, long)]
        verbose: bool,
    },
}

pub struct Stats {
    current_block_height: u64,
    total_validators: u64,
    active_validators: u64,
    inactive_validators: u64,
    jailed_validators: u64,
    tombstoned_validators: u64,
    disabled_validators: u64,
}

impl ChainCmd {
    pub async fn print_chain_params(&self, app: &mut App) -> Result<()> {
        let mut oblivious_client = app.oblivious_client().await?;

        let params = oblivious_client
            .chain_parameters(tonic::Request::new(ChainParametersRequest {
                chain_id: "".to_string(),
            }))
            .await?
            .into_inner()
            .chain_parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?;

        println!("Chain Parameters:");
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table
            .set_header(vec!["", ""])
            .add_row(vec!["Chain ID", &params.chain_id])
            .add_row(vec![
                "Epoch Duration",
                &format!("{}", params.epoch_duration),
            ])
            .add_row(vec![
                "Unbonding Epochs",
                &format!("{}", params.unbonding_epochs),
            ])
            .add_row(vec![
                "Active Validator Limit",
                &format!("{}", params.active_validator_limit),
            ])
            .add_row(vec![
                "Base Reward Rate (bps^2)",
                &format!("{}", params.base_reward_rate),
            ])
            .add_row(vec![
                "Slashing Penalty (Misbehavior) (bps^2)",
                &format!("{}", params.slashing_penalty_misbehavior),
            ])
            .add_row(vec![
                "Slashing Penalty (Downtime) (bps^2)",
                &format!("{}", params.slashing_penalty_downtime),
            ])
            .add_row(vec![
                "Signed Blocks Window (blocks)",
                &format!("{}", params.signed_blocks_window_len),
            ])
            .add_row(vec![
                "Missed Blocks Max",
                &format!("{}", params.missed_blocks_maximum),
            ])
            .add_row(vec![
                "Proposal Deposit Amount (upenumbra)",
                &format!("{}", params.proposal_deposit_amount),
            ])
            .add_row(vec!["IBC Enabled", &format!("{}", params.ibc_enabled)])
            .add_row(vec![
                "Inbound ICS-20 Enabled",
                &format!("{}", params.inbound_ics20_transfers_enabled),
            ])
            .add_row(vec![
                "Outbound ICS-20 Enabled",
                &format!("{}", params.outbound_ics20_transfers_enabled),
            ]);

        println!("{table}");

        Ok(())
    }

    pub async fn get_stats(&self, app: &mut App) -> Result<Stats> {
        use penumbra_proto::client::v1alpha1::ValidatorInfoRequest;

        let mut client = app.oblivious_client().await?;

        // TODO: is it possible to use the TendermintProxyService instead here??
        let info = client
            .info(InfoRequest {
                version: "".to_string(),
                block_version: 0,
                p2p_version: 0,
            })
            .await?
            .into_inner();

        let current_block_height = info.last_block_height;
        let chain_params = client
            .chain_parameters(tonic::Request::new(ChainParametersRequest {
                chain_id: "".to_string(),
            }))
            .await?
            .into_inner()
            .chain_parameters
            .ok_or_else(|| anyhow::anyhow!("empty ChainParametersResponse message"))?;

        // Fetch validators.
        let validators = client
            .validator_info(ValidatorInfoRequest {
                show_inactive: true,
                chain_id: chain_params.chain_id,
            })
            .await?
            .into_inner()
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<validator::Info>, _>>()?;

        let total_validators = validators.len() as u64;
        let active_validators = validators
            .iter()
            .filter(|v| v.status.state == validator::State::Active)
            .count() as u64;
        let inactive_validators = validators
            .iter()
            .filter(|v| v.status.state == validator::State::Inactive)
            .count() as u64;
        let jailed_validators = validators
            .iter()
            .filter(|v| v.status.state == validator::State::Jailed)
            .count() as u64;
        let tombstoned_validators = validators
            .iter()
            .filter(|v| v.status.state == validator::State::Tombstoned)
            .count() as u64;
        let disabled_validators = validators
            .iter()
            .filter(|v| v.status.state == validator::State::Disabled)
            .count() as u64;

        Ok(Stats {
            current_block_height,
            total_validators,
            active_validators,
            inactive_validators,
            jailed_validators,
            tombstoned_validators,
            disabled_validators,
        })
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            ChainCmd::Params => {
                self.print_chain_params(app).await?;
            }
            // TODO: we could implement this as an RPC call using the metrics
            // subsystems once #829 is complete
            // OR (hdevalence): fold it into pcli q
            ChainCmd::Info { verbose } => {
                if *verbose {
                    self.print_chain_params(app).await?;
                }

                let stats = self.get_stats(app).await?;

                println!("Chain Info:");
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table
                    .set_header(vec!["", ""])
                    .add_row(vec![
                        "Current Block Height",
                        &format!("{}", stats.current_block_height),
                    ])
                    .add_row(vec![
                        "Total Validators",
                        &format!("{}", stats.total_validators),
                    ])
                    .add_row(vec![
                        "Active Validators",
                        &format!("{}", stats.active_validators),
                    ])
                    .add_row(vec![
                        "Inactive Validators",
                        &format!("{}", stats.inactive_validators),
                    ])
                    .add_row(vec![
                        "Jailed Validators",
                        &format!("{}", stats.jailed_validators),
                    ])
                    .add_row(vec![
                        "Tombstoned Validators",
                        &format!("{}", stats.tombstoned_validators),
                    ])
                    .add_row(vec![
                        "Disabled Validators",
                        &format!("{}", stats.disabled_validators),
                    ]);

                println!("{table}");
            }
        };

        Ok(())
    }
}
