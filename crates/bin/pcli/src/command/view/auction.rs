use anyhow::Result;
use comfy_table::{presets, Cell, ContentArrangement, Table};
use penumbra_sdk_auction::auction::dutch::DutchAuction;
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_proto::{core::component::auction::v1 as pb_auction, DomainType, Name};
use penumbra_sdk_view::ViewClient;

use crate::command::query::auction::render_dutch_auction;

#[derive(Debug, clap::Args)]
pub struct AuctionCmd {
    #[clap(long)]
    /// If set, includes the inactive auctions as well.
    pub include_inactive: bool,
    /// If set, make the view server query an RPC and pcli render the full auction state
    #[clap(long, default_value_t = true)]
    pub query_latest_state: bool,
}

impl AuctionCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec(
        &self,
        view_client: &mut impl ViewClient,
        _fvk: &FullViewingKey,
    ) -> Result<()> {
        let auctions: Vec<(
            penumbra_sdk_auction::auction::AuctionId,
            penumbra_sdk_view::SpendableNoteRecord,
            u64,
            Option<pbjson_types::Any>,
            Vec<penumbra_sdk_dex::lp::position::Position>,
        )> = view_client
            .auctions(None, self.include_inactive, self.query_latest_state)
            .await?;

        for (auction_id, _, local_seq, maybe_auction_state, positions) in auctions.into_iter() {
            if let Some(pb_auction_state) = maybe_auction_state {
                if pb_auction_state.type_url == pb_auction::DutchAuction::type_url() {
                    let dutch_auction = DutchAuction::decode(pb_auction_state.value)
                        .expect("no deserialization error");
                    let asset_cache = view_client.assets().await?;
                    render_dutch_auction(
                        &asset_cache,
                        &dutch_auction,
                        Some(local_seq),
                        positions.get(0).cloned(),
                    )
                    .await
                    .expect("no rendering errors");
                } else {
                    unimplemented!("only supporting dutch auctions at the moment, come back later");
                }
            } else {
                let position_ids: Vec<String> = positions
                    .into_iter()
                    .map(|lp: penumbra_sdk_dex::lp::position::Position| format!("{}", lp.id()))
                    .collect();

                let mut auction_table = Table::new();
                auction_table.load_preset(presets::ASCII_FULL);
                auction_table
                    .set_header(vec!["Auction id", "LPs"])
                    .set_content_arrangement(ContentArrangement::DynamicFullWidth)
                    .add_row(vec![
                        Cell::new(&auction_id).set_delimiter('.'),
                        Cell::new(format!("{:?}", position_ids))
                            .set_alignment(comfy_table::CellAlignment::Center),
                    ]);

                println!("{auction_table}");
            }
        }
        Ok(())
    }
}
