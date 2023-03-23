use std::pin::Pin;

use anyhow::{Context, Result};
use comfy_table::{presets, Table};
use futures::{Future, FutureExt, Stream, StreamExt, TryStreamExt};
use penumbra_crypto::dex::{
    lp::{position::Metadata, Reserves},
    BatchSwapOutputData, TradingPair,
};
use penumbra_proto::client::v1alpha1::{
    specific_query_service_client::SpecificQueryServiceClient, BatchSwapOutputDataRequest,
    LiquidityPositionsRequest, StubCpmmReservesRequest,
};
use penumbra_view::ViewClient;
use tonic::transport::Channel;

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
        #[clap(long)]
        height: u64,
        /// The trading pair to query for batch outputs.
        trading_pair: TradingPair,
    },
    /// Display information about liquidity positions known to the chain.
    LiquidityPositions {
        /// Display closed and withdrawn liquidity positions.
        #[clap(default_value_t = true)]
        only_open: bool,
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
                chain_id: app.view().chain_params().await?.chain_id,
            })
            .await?
            .into_inner()
            .try_into()
            .context("cannot parse stub CPMM reserves data")?;
        println!("Constant-Product Market Maker Reserves:");
        let mut table = Table::new();
        // TODO: use oblivious query service instead of view service
        let view_client: &mut dyn ViewClient = app.view.as_mut().unwrap();
        let asset_cache = view_client.assets().await?;
        let asset_1 = asset_cache
            .get(&trading_pair.asset_1())
            .map(|base_denom| {
                let display_denom = base_denom.best_unit_for(reserves_data.r1);
                (
                    format!("{display_denom}"),
                    display_denom.format_value(reserves_data.r1),
                )
            })
            .unwrap_or_else(|| {
                (
                    format!("{}", trading_pair.asset_1()),
                    reserves_data.r1.to_string(),
                )
            });
        let asset_2 = asset_cache
            .get(&trading_pair.asset_2())
            .map(|base_denom| {
                let display_denom = base_denom.best_unit_for(reserves_data.r2);
                (
                    format!("{display_denom}"),
                    display_denom.format_value(reserves_data.r2),
                )
            })
            .unwrap_or_else(|| {
                (
                    format!("{}", trading_pair.asset_2()),
                    reserves_data.r2.to_string(),
                )
            });
        table.load_preset(presets::NOTHING);
        table
            .set_header(vec!["Denomination", "Reserve Amount"])
            .add_row(vec![asset_1.0, asset_1.1])
            .add_row(vec![asset_2.0, asset_2.1]);

        println!("{table}");

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
                chain_id: app.view().chain_params().await?.chain_id,
            })
            .await?
            .into_inner()
            .try_into()
            .context("cannot parse batch swap output data")
    }

    pub async fn get_liquidity_positions(
        &self,
        mut client: SpecificQueryServiceClient<Channel>,
        only_open: bool,
        chain_id: String,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<Pin<Box<dyn Stream<Item = Result<Metadata>> + Send + 'static>>>,
                > + Send
                + 'static,
        >,
    > {
        async move {
            let stream = client.liquidity_positions(LiquidityPositionsRequest {
                only_open,
                chain_id,
            });
            let stream = stream.await?.into_inner();

            Ok(stream
                .map_err(|e| anyhow::anyhow!("error fetching liquidity positions: {}", e))
                .and_then(|msg| async move {
                    msg.data
                        .ok_or(anyhow::anyhow!(
                            "missing liquidity position in response data"
                        ))
                        .map(|data| Metadata::try_from(data))?
                })
                .boxed())
        }
        .boxed()
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

                // TODO: use oblivious query service instead of view service
                let view_client: &mut dyn ViewClient = app.view.as_mut().unwrap();
                let asset_cache = view_client.assets().await?;
                let asset_1 = asset_cache
                    .get(&trading_pair.asset_1())
                    .map(|base_denom| {
                        let display_denom = base_denom.best_unit_for(
                            std::cmp::max(outputs.delta_1, outputs.lambda_1_1 + outputs.lambda_1_2)
                                .into(),
                        );
                        (
                            format!("{display_denom}"),
                            display_denom.format_value(outputs.delta_1.into()),
                            display_denom
                                .format_value((outputs.lambda_1_1 + outputs.lambda_1_2).into()),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            format!("{}", trading_pair.asset_1()),
                            outputs.delta_1.to_string(),
                            (outputs.lambda_1_1 + outputs.lambda_1_2).to_string(),
                        )
                    });
                let asset_2 = asset_cache
                    .get(&trading_pair.asset_2())
                    .map(|base_denom| {
                        let display_denom = base_denom.best_unit_for(
                            std::cmp::max(outputs.delta_2, outputs.lambda_2_1 + outputs.lambda_2_2)
                                .into(),
                        );
                        (
                            format!("{display_denom}"),
                            display_denom.format_value(outputs.delta_2.into()),
                            display_denom
                                .format_value((outputs.lambda_2_1 + outputs.lambda_2_2).into()),
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            format!("{}", trading_pair.asset_2()),
                            outputs.delta_2.to_string(),
                            (outputs.lambda_2_1 + outputs.lambda_2_2).to_string(),
                        )
                    });

                println!("Batch Swap Outputs for height {}:", outputs.height);
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table
                    .set_header(vec!["Denomination", "Input Amount", "Output Amount"])
                    .add_row(vec![asset_1.0, asset_1.1, asset_1.2])
                    .add_row(vec![asset_2.0, asset_2.1, asset_2.2]);

                println!("{table}");
            }
            DexCmd::LiquidityPositions { only_open } => {
                let client = app.specific_client().await.unwrap();
                // TODO: use oblivious query service instead of view service
                let view_client: &mut dyn ViewClient = app.view.as_mut().unwrap();
                let chain_id = view_client.chain_params().await?.chain_id;
                let asset_cache = view_client.assets().await?;

                let mut positions_stream = self
                    .get_liquidity_positions(client, *only_open, chain_id)
                    .await
                    .await?;

                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table.set_header(vec![
                    "Trading Pair",
                    "State",
                    "Reserves",
                    "Trading Function",
                ]);

                while let Ok(position) =
                    positions_stream.next().await.transpose()?.ok_or_else(|| {
                        anyhow::anyhow!("view service did not return liquidity position")
                    })
                {
                    let trading_pair = position.position.phi.pair;
                    let asset_1 = asset_cache
                        .get(&trading_pair.asset_1())
                        .map(|bd| format!("{bd}"))
                        .unwrap_or("unknown".to_string());
                    let asset_2 = asset_cache
                        .get(&trading_pair.asset_2())
                        .map(|bd| format!("{bd}"))
                        .unwrap_or("unknown".to_string());
                    table.add_row(vec![
                        format!("({}, {})", asset_1, asset_2),
                        position.state.to_string(),
                        format!("({}, {})", position.reserves.r1, position.reserves.r2),
                        format!(
                            "p: {} q: {} fee: {}",
                            position.position.phi.component.p,
                            position.position.phi.component.q,
                            position.position.phi.component.fee
                        ),
                    ]);
                }

                println!("{table}");
            }
        };

        Ok(())
    }
}
