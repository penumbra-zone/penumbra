use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::FullViewingKey;
use penumbra_view::ViewClient;

#[derive(Debug, clap::Args)]
pub struct TransactionsCmd {
    #[clap(short, long)]
    pub start_height: Option<u64>,
    #[clap(short, long)]
    pub end_height: Option<u64>,
}

impl TransactionsCmd {
    pub fn needs_sync(&self) -> bool {
        true
    }

    pub async fn exec<V: ViewClient>(&self, fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        // Initialize the table

        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        let txs = view
            .transactions(fvk.hash(), self.start_height, self.end_height)
            .await?;

        table.set_header(vec!["Block Height", "Transaction Hash"]);

        for tx in txs {
            table.add_row(vec![
                format!("{}", u128::from(tx.0)),
                format!("{:?}", hex::encode(tx.1)),
            ]);
        }

        println!("{}", table);

        Ok(())
    }
}
