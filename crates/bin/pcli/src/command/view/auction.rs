use anyhow::Result;
use penumbra_keys::FullViewingKey;
use penumbra_view::ViewClient;

#[derive(Debug, clap::Args)]
pub struct AuctionCmd {
    #[clap(long)]
    /// If set, includes the inactive auctions as well.
    pub include_inactive: bool,
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
        let auctions = view_client
            .auctions(None, self.include_inactive, false)
            .await?;

        auctions.iter().for_each(|(id, snr, _, _)| {
            println!("{id:?} {}", snr.note.amount());
        });
        Ok(())
    }
}
