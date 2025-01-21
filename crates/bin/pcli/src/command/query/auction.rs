use crate::command::utils::render_positions;
use crate::App;
use clap::Subcommand;
use comfy_table::{presets, Table};
use comfy_table::{Cell, ContentArrangement};
use penumbra_sdk_asset::asset::Cache;
use penumbra_sdk_asset::Value;
use penumbra_sdk_auction::auction::dutch::DutchAuction;
use penumbra_sdk_auction::auction::AuctionId;
use penumbra_sdk_dex::lp::position::{self, Position};
use penumbra_sdk_num::fixpoint::U128x128;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::core::component::auction::v1 as pb_auction;
use penumbra_sdk_proto::core::component::auction::v1::query_service_client::QueryServiceClient as AuctionQueryServiceClient;
use penumbra_sdk_proto::core::component::auction::v1::AuctionStateByIdRequest;
use penumbra_sdk_proto::core::component::dex::v1::query_service_client::QueryServiceClient as DexQueryServiceClient;
use penumbra_sdk_proto::core::component::dex::v1::LiquidityPositionByIdRequest;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_proto::Name;
use penumbra_sdk_view::ViewClient;

#[derive(Debug, Subcommand)]
pub enum AuctionCmd {
    /// Commands related to Dutch auctions
    Dutch {
        #[clap(index = 1)]
        auction_id: AuctionId,
    },
}

impl AuctionCmd {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        match self {
            AuctionCmd::Dutch { auction_id } => {
                let auction_id = auction_id.clone();
                let mut auction_client = AuctionQueryServiceClient::new(app.pd_channel().await?);
                let rsp = auction_client
                    .auction_state_by_id(AuctionStateByIdRequest {
                        id: Some(auction_id.into()),
                    })
                    .await?
                    .into_inner();

                let pb_auction_state = rsp
                    .auction
                    .ok_or_else(|| anyhow::anyhow!("auction state is missing!"))?;

                if pb_auction_state.type_url == pb_auction::DutchAuction::type_url() {
                    let dutch_auction = DutchAuction::decode(pb_auction_state.value)?;
                    let position = if let Some(position_id) = dutch_auction.state.current_position {
                        let mut dex_client = DexQueryServiceClient::new(app.pd_channel().await?);
                        let position: Position = dex_client
                            .liquidity_position_by_id(LiquidityPositionByIdRequest {
                                position_id: Some(position_id.into()),
                            })
                            .await?
                            .into_inner()
                            .data
                            .expect("a position should exist")
                            .try_into()
                            .expect("no decoding error");
                        Some(position)
                    } else {
                        None
                    };

                    let asset_cache = app.view().assets().await?;

                    render_dutch_auction(&asset_cache, &dutch_auction, None, position).await?;
                } else {
                    unimplemented!("only supporting dutch auctions at the moment, come back later");
                }
            }
        }
        Ok(())
    }
}

pub async fn render_dutch_auction(
    asset_cache: &Cache,
    dutch_auction: &DutchAuction,
    local_view: Option<u64>,
    position: Option<Position>,
) -> anyhow::Result<()> {
    let auction_id = dutch_auction.description.id();
    println!("dutch auction with id {auction_id:?}:");

    let initial_input = dutch_auction.description.input;
    let input_id = initial_input.asset_id;
    let output_id = dutch_auction.description.output_id;

    let initial_input_amount = U128x128::from(initial_input.amount);
    let min_output = U128x128::from(dutch_auction.description.min_output);
    let max_output = U128x128::from(dutch_auction.description.max_output);
    let start_price = (max_output / initial_input_amount).expect("the input is always nonzero");
    let end_price = (min_output / initial_input_amount).expect("the input is always nonzero");

    let maybe_id = dutch_auction.state.current_position;

    let (position_input_reserve, position_output_reserve) = position.as_ref().map_or_else(
        || (Amount::zero(), Amount::zero()),
        |lp| {
            (
                lp.reserves_for(input_id)
                    .expect("lp doesn't have reserves for input asset"),
                lp.reserves_for(output_id)
                    .expect("lp doesn't have reserves for output asset"),
            )
        },
    );

    let auction_input_reserves = Value {
        amount: position_input_reserve + dutch_auction.state.input_reserves,
        asset_id: input_id,
    };
    let auction_output_reserves = Value {
        amount: position_output_reserve + dutch_auction.state.output_reserves,
        asset_id: output_id,
    };

    let start_height = dutch_auction.description.start_height;
    let end_height = dutch_auction.description.end_height;

    let mut auction_table = Table::new();
    auction_table.load_preset(presets::UTF8_FULL);
    auction_table
        .set_header(vec![
            "Auction id",
            "State",
            "Height range",
            "# steps",
            "Start price",
            "End price",
            "Input",
            "Balance",
            "Has lp?",
        ])
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .add_row(vec![
            Cell::new(truncate_auction_id(&auction_id)).set_delimiter('.'),
            Cell::new(render_sequence(dutch_auction.state.sequence, local_view)),
            Cell::new(format!("{start_height} -> {end_height}")),
            Cell::new(dutch_auction.description.step_count.to_string()),
            Cell::new(format!("{}", start_price)),
            Cell::new(format!("{}", end_price)),
            Cell::new(initial_input.format(asset_cache)),
            Cell::new(format!(
                "({}, {})",
                &auction_input_reserves.format(asset_cache),
                &auction_output_reserves.format(asset_cache)
            )),
            Cell::new(format!("{}", render_position_id(&maybe_id)))
                .set_alignment(comfy_table::CellAlignment::Center),
        ]);

    if let Some(lp) = position {
        auction_table.add_row(vec![Cell::new(format!(
            "{}",
            render_positions(asset_cache, &[lp])
        ))]);
    }

    println!("{auction_table}");
    Ok(())
}

fn render_sequence(state: u64, local_seq: Option<u64>) -> String {
    let main = if state == 0 {
        format!("Opened")
    } else if state == 1 {
        format!("Closed")
    } else {
        format!("Withdrawn (seq={state})")
    };

    if let Some(local_seq) = local_seq {
        format!("{main} (local_seq={local_seq})")
    } else {
        main
    }
}

fn truncate_auction_id(asset_id: &AuctionId) -> String {
    let input = format!("{asset_id:?}");
    let prefix_len = 16;
    if input.len() > prefix_len {
        format!("{}...", &input[..prefix_len])
    } else {
        input
    }
}

fn render_position_id(maybe_id: &Option<position::Id>) -> String {
    let input = maybe_id.map_or_else(|| format!("x"), |_| format!("✓"));
    let prefix_len = 6;
    if input.len() > prefix_len {
        format!("{}..", &input[..prefix_len])
    } else {
        input
    }
}
