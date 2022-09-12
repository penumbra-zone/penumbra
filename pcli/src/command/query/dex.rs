use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use penumbra_crypto::dex::{lp::Reserves, BatchSwapOutputData, TradingPair};
use penumbra_proto::client::specific::{BatchSwapOutputDataRequest, StubCpmmReservesRequest};

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

    pub async fn get_batch_outputs(
        &self,
        app: &mut App,
        height: &u64,
        trading_pair: &TradingPair,
    ) -> Result<BatchSwapOutputData> {
        let mut client = app.specific_client().await?;
        client
            .batch_swap_output_data(BatchSwapOutputDataRequest {
                height: *height,
                trading_pair: Some((*trading_pair).into()),
            })
            .await?
            .into_inner()
            .try_into()
            .context("cannot parse batch swap output data")
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

                println!(
                    "Batch Swap Output status was: {}",
                    if outputs.success {
                        "Success"
                    } else {
                        "Failure"
                    }
                );
                println!("Batch Swap Outputs for height {}:", outputs.height);
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table
                    .set_header(vec!["Asset ID", "Input Amount", "Output Amount"])
                    .add_row(vec![
                        outputs.trading_pair.asset_1().to_string(),
                        outputs.delta_1.to_string(),
                        outputs.lambda_1.to_string(),
                    ])
                    .add_row(vec![
                        outputs.trading_pair.asset_2().to_string(),
                        outputs.delta_2.to_string(),
                        outputs.lambda_2.to_string(),
                    ]);

                println!("{}", table);
            }
        };

        Ok(())
    }
}
