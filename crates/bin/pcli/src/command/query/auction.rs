use crate::App;
use clap::Subcommand;
use comfy_table::{presets, Table};
use penumbra_auction::auction::dutch::DutchAuction;
use penumbra_auction::auction::AuctionId;
use penumbra_proto::core::component::auction::v1alpha1 as pb_auction;
use penumbra_proto::core::component::auction::v1alpha1::query_service_client::QueryServiceClient;
use penumbra_proto::core::component::auction::v1alpha1::AuctionStateByIdRequest;
use penumbra_proto::DomainType;
use penumbra_proto::Name;

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
                let mut client = QueryServiceClient::new(app.pd_channel().await?);
                let rsp = client
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
                    println!("dutch auction with id {auction_id:?}");

                    let mut table = Table::new();
                    table.load_preset(presets::NOTHING);
                    table
                        .set_header(vec![
                            "Auction Id",
                            "State",
                            "Start height",
                            "End height",
                            "Step count",
                        ]) // TODO: make this more useful
                        .add_row(vec![
                            &auction_id.to_string(),
                            &render_state(dutch_auction.state.sequence),
                            &dutch_auction.description.start_height.to_string(),
                            &dutch_auction.description.end_height.to_string(),
                            &dutch_auction.description.step_count.to_string(),
                        ]);
                    println!("{table}");
                } else {
                    unimplemented!("only supporting dutch auctions at the moment, come back later");
                }
            }
        }
        Ok(())
    }
}

fn render_state(state: u64) -> String {
    if state == 0 {
        format!("Opened")
    } else if state == 1 {
        format!("Closed")
    } else {
        format!("Withdrawn (seq={state})")
    }
}
