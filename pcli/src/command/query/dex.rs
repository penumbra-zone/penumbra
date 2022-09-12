use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_chain::Epoch;
use penumbra_component::stake::validator;
use penumbra_crypto::dex::{lp::Reserves, BatchSwapOutputData, TradingPair};
use penumbra_proto::client::specific::StubCpmmReservesRequest;
use penumbra_view::ViewClient;

use crate::App;

#[derive(Debug, clap::Subcommand)]
pub enum DexCmd {
    /// Display information about constant-pair market maker reserves.
    CPMMReserves {
        /// The trading pair to query for CPMM Reserves.
        trading_pair: TradingPair,
    },
    /// Display information about a specific trading pair & height's batch swap.
    BatchOutputs {
        /// The height to query for batch outputs.
        #[clap(short, long)]
        height: u64,
        /// The trading pair to query for batch outputs.
        trading_pair: TradingPair,
    },
}

impl DexCmd {
    pub async fn print_cpmm_reserves(
        &self,
        app: &mut App,
        trading_pair: &TradingPair,
    ) -> Result<()> {
        let mut client = app.specific_client().await?;
        let reserves_data: Reserves = client
            .stub_cpmm_reserves(StubCpmmReservesRequest {
                trading_pair: Some((*trading_pair).into()),
            })
            .await?
            .into_inner()
            .try_into()
            .context("cannot parse stub CPMM reserves data")?;
        println!("Constant-Product Market Maker Reserves:");
        let mut table = Table::new();
        table.load_preset(presets::NOTHING);
        table
            .set_header(vec!["Asset ID", "Reserve Amount"])
            .add_row(vec![
                trading_pair.asset_1().to_string(),
                reserves_data.r1.to_string(),
            ])
            .add_row(vec![
                trading_pair.asset_2().to_string(),
                reserves_data.r2.to_string(),
            ]);

        println!("{}", table);

        Ok(())
    }

    // pub async fn get_stats(&self, app: &mut App) -> Result<Stats> {
    //     use penumbra_proto::client::oblivious::ValidatorInfoRequest;

    //     let mut client = app.oblivious_client().await?;
    //     let fvk = &app.fvk;
    //     let view: &mut dyn ViewClient = &mut app.view;

    //     let current_block_height = view.status(fvk.hash()).await?.sync_height;
    //     let chain_params = view.chain_params().await?;

    //     let epoch_duration = chain_params.epoch_duration;
    //     let current_epoch = Epoch::from_height(current_block_height, epoch_duration).index;

    //     // Fetch validators.
    //     let validators = client
    //         .validator_info(ValidatorInfoRequest {
    //             show_inactive: true,
    //             chain_id: chain_params.chain_id,
    //         })
    //         .await?
    //         .into_inner()
    //         .try_collect::<Vec<_>>()
    //         .await?
    //         .into_iter()
    //         .map(TryInto::try_into)
    //         .collect::<Result<Vec<validator::Info>, _>>()?;

    //     let total_validators = validators.len() as u64;
    //     let active_validators = validators
    //         .iter()
    //         .filter(|v| v.status.state == validator::State::Active)
    //         .count() as u64;
    //     let inactive_validators = validators
    //         .iter()
    //         .filter(|v| v.status.state == validator::State::Inactive)
    //         .count() as u64;
    //     let jailed_validators = validators
    //         .iter()
    //         .filter(|v| v.status.state == validator::State::Jailed)
    //         .count() as u64;
    //     let tombstoned_validators = validators
    //         .iter()
    //         .filter(|v| v.status.state == validator::State::Tombstoned)
    //         .count() as u64;
    //     let disabled_validators = validators
    //         .iter()
    //         .filter(|v| v.status.state == validator::State::Disabled)
    //         .count() as u64;

    //     Ok(Stats {
    //         current_block_height,
    //         current_epoch,
    //         total_validators,
    //         active_validators,
    //         inactive_validators,
    //         jailed_validators,
    //         tombstoned_validators,
    //         disabled_validators,
    //     })
    // }
    pub async fn get_batch_outputs(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<BatchSwapOutputData> {
        Err(anyhow::anyhow!("not implemented"))
        // let mut client = app.specific_client().await?;
        // let output_data: BatchSwapOutputData = client
        //     .batch_swap_output_data(BatchSwapOutputDataRequest {
        //         height: swap_nft_record.height_created,
        //         trading_pair: Some(swap_plaintext.trading_pair.into()),
        //     })
        //     .await?
        //     .into_inner()
        //     .try_into()
        //     .context("cannot parse batch swap output data")?;
    }

    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            DexCmd::CPMMReserves { trading_pair } => {
                self.print_cpmm_reserves(app, trading_pair).await?;
            }
            DexCmd::BatchOutputs {
                height,
                trading_pair,
            } => {
                let outputs = self.get_batch_outputs(app, height, trading_pair).await?;

                println!("Batch Outputs:");
                // let mut table = Table::new();
                // table.load_preset(presets::NOTHING);
                // table
                //     .set_header(vec!["", ""])
                //     .add_row(vec![
                //         "Current Block Height",
                //         &format!("{}", stats.current_block_height),
                //     ])
                //     .add_row(vec!["Current Epoch", &format!("{}", stats.current_epoch)])
                //     .add_row(vec![
                //         "Total Validators",
                //         &format!("{}", stats.total_validators),
                //     ])
                //     .add_row(vec![
                //         "Active Validators",
                //         &format!("{}", stats.active_validators),
                //     ])
                //     .add_row(vec![
                //         "Inactive Validators",
                //         &format!("{}", stats.inactive_validators),
                //     ])
                //     .add_row(vec![
                //         "Jailed Validators",
                //         &format!("{}", stats.jailed_validators),
                //     ])
                //     .add_row(vec![
                //         "Tombstoned Validators",
                //         &format!("{}", stats.tombstoned_validators),
                //     ])
                //     .add_row(vec![
                //         "Disabled Validators",
                //         &format!("{}", stats.disabled_validators),
                //     ]);

                // println!("{}", table);
            }
        };

        Ok(())
    }
}
