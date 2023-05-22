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
            .transaction_info(self.start_height, self.end_height)
            .await?;

        table.set_header(vec!["Block Height", "Transaction Hash"]);

        for tx_info in txs {
            table.add_row(vec![
                format!("{}", tx_info.height),
                format!("{}", hex::encode(&tx_info.id)),
            ]);
        }

        println!("{table}");

        Ok(())
    }
}
