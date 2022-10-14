use anyhow::Result;
use comfy_table::{presets, Table};
use penumbra_crypto::FullViewingKey;
use penumbra_view::ViewClient;

#[derive(Debug, clap::Args)]
pub struct TransactionHashesCmd {
    #[clap(short, long)]
    pub start_height: Option<u64>,
    #[clap(short, long)]
    pub end_height: Option<u64>,
}

impl TransactionHashesCmd {
    pub fn offline(&self) -> bool {
        false
    }

    pub async fn exec<V: ViewClient>(&self, _fvk: &FullViewingKey, view: &mut V) -> Result<()> {
        // Initialize the table

        let mut table = Table::new();
        table.load_preset(presets::NOTHING);

        let txs = view
            .transaction_hashes(self.start_height, self.end_height)
            .await?;

        table.set_header(vec!["Block Height", "Transaction Hash"]);

        for tx in txs {
            table.add_row(vec![
                format!("{}", u128::from(tx.0)),
                format!("{}", hex::encode(tx.1)),
            ]);
        }

        println!("{}", table);

        Ok(())
    }
}
