use crate::App;
use clap::Subcommand;
use comfy_table::{presets, Table};
use penumbra_auction::auction::dutch::DutchAuction;
use penumbra_auction::auction::AuctionId;
use penumbra_proto::core::component::auction::v1alpha1::query_service_client::QueryServiceClient;
use penumbra_proto::core::component::auction::v1alpha1::AuctionStateByIdRequest;
use penumbra_proto::Name;

#[derive(Debug, Subcommand)]
pub enum AuctionCmd {
    /// Commands related to Dutch auctions
    #[clap(display_order = 100, subcommand)]
    Dutch(DutchCmd),
}

/// Commands related to querying Dutch auctions.
#[derive(Debug, Subcommand)]
pub enum DutchCmd {
    #[clap(display_order = 100, name = "state")]
    State {
        #[clap(long, display_order = 100)]
        auction_id: AuctionId,
    },
}

impl DutchCmd {
    pub async fn exec(&self, app: &mut App) -> anyhow::Result<()> {
        match self {
            DutchCmd::State { auction_id } => {
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
                use penumbra_proto::core::component::auction::v1alpha1 as pb_auction;
                use penumbra_proto::DomainType;

                if pb_auction_state.type_url == pb_auction::DutchAuction::type_url() {
                    let auction_state = DutchAuction::decode(pb_auction_state.value)?;
                    println!("dutch auction with id {auction_id:?}");

                    let mut table = Table::new();
                    table.load_preset(presets::NOTHING);
                    table
                        .set_header(vec!["", ""])
                        .add_row(vec!["Auction Id", &auction_id.to_string()])
                        .add_row(vec![
                            "State",
                            &format!("Opened ({})", auction_state.state.sequence),
                        ]); // TODO: render state string
                    println!("{table}");
                } else {
                    unimplemented!("only supporting dutch auctions at the moment, come back later");
                }
            }
        }
        Ok(())
    }
}
