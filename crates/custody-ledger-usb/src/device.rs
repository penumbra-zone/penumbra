use ledger_lib::{Filters, LedgerProvider, Transport};

pub async fn main() -> anyhow::Result<()> {
    let mut provider = LedgerProvider::init().await;
    let devices = provider.list(Filters::Any).await?;
    dbg!(&devices);
    println!("main");
    Ok(())
}
