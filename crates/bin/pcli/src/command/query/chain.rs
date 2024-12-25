use anyhow::{anyhow, Context, Result};
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_sdk_app::params::AppParameters;
use penumbra_sdk_proto::{
    core::{
        app::v1::{
            query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
        },
        component::{
            sct::v1::{
                query_service_client::QueryServiceClient as SctQueryServiceClient,
                EpochByHeightRequest,
            },
            stake::v1::{
                query_service_client::QueryServiceClient as StakeQueryServiceClient,
                ValidatorInfoRequest,
            },
        },
    },
    util::tendermint_proxy::v1::{
        tendermint_proxy_service_client::TendermintProxyServiceClient, AbciQueryRequest,
        GetStatusRequest,
    },
    Message,
};
use penumbra_sdk_stake::validator;

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
    DetectDesync,
}

pub struct Stats {
    current_block_height: u64,
    current_epoch: u64,
    total_validators: u64,
    active_validators: u64,
    inactive_validators: u64,
    jailed_validators: u64,
    tombstoned_validators: u64,
    disabled_validators: u64,
}

impl ChainCmd {
    pub async fn print_app_params(&self, app: &mut App) -> Result<()> {
        let mut client = AppQueryServiceClient::new(app.pd_channel().await?);
        let params: AppParameters = client
            .app_parameters(tonic::Request::new(AppParametersRequest {}))
            .await?
            .into_inner()
            .app_parameters
            .ok_or_else(|| anyhow::anyhow!("empty AppParametersResponse message"))?
            .try_into()?;

        // Use serde-json to pretty print the params
        let params_json = serde_json::to_string_pretty(&params)?;
        println!("{}", params_json);

        Ok(())
    }

    pub async fn get_stats(&self, app: &mut App) -> Result<Stats> {
        let channel = app.pd_channel().await?;

        let mut client = TendermintProxyServiceClient::new(channel.clone());
        let current_block_height = client
            .get_status(GetStatusRequest::default())
            .await?
            .into_inner()
            .sync_info
            .ok_or_else(|| anyhow!("missing sync_info"))?
            .latest_block_height;

        let mut client = SctQueryServiceClient::new(channel.clone());
        let current_epoch: u64 = client
            .epoch_by_height(tonic::Request::new(EpochByHeightRequest {
                height: current_block_height.clone(),
            }))
            .await?
            .into_inner()
            .epoch
            .context("failed to find EpochByHeight message")?
            .index;

        // Fetch validators.
        let mut client = StakeQueryServiceClient::new(channel.clone());
        let validators = client
            .validator_info(ValidatorInfoRequest {
                show_inactive: true,
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
            current_epoch,
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
            ChainCmd::DetectDesync => {
                let mut client = TendermintProxyServiceClient::new(app.pd_channel().await?);
                let status = client
                    .get_status(GetStatusRequest::default())
                    .await?
                    .into_inner()
                    .sync_info
                    .ok_or_else(|| anyhow!("missing sync_info"))?;

                let mut app_client = AppQueryServiceClient::new(app.pd_channel().await?);
                let params = app_client
                    .app_parameters(AppParametersRequest {})
                    .await?
                    .into_inner()
                    .app_parameters
                    .unwrap();
                let chain_id = params.chain_id;

                let height = status.latest_block_height as i64;

                let response = client
                    .abci_query(AbciQueryRequest {
                        data: b"sct/block_manager/block_height".to_vec(),
                        path: "state/key".to_string(),
                        height,
                        prove: false,
                    })
                    .await?
                    .into_inner();

                let raw_height_response = response.value;
                let height_response: u64 = Message::decode(&raw_height_response[..])
                    .map_err(|e| anyhow!("failed to decode height response: {}", e))?;

                println!("chain_id: {}", chain_id);
                println!("queried height: {}", height);
                println!("height response: {}", height_response);
                if height == height_response as i64 {
                    println!(
                        "Unaffected. No action item. The full node internal state version tracks the block height."
                    );
                } else {
                    println!("Affected. The full node chain state is corrupted, please resync your node.");
                }
            }
            ChainCmd::Params => {
                self.print_app_params(app).await?;
            }
            // TODO: we could implement this as an RPC call using the metrics
            // subsystems once #829 is complete
            // OR (hdevalence): fold it into pcli q
            ChainCmd::Info { verbose } => {
                if *verbose {
                    self.print_app_params(app).await?;
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
                    .add_row(vec!["Current Epoch", &format!("{}", stats.current_epoch)])
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
