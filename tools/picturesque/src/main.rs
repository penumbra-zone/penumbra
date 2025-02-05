use picturesque::run_devnet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_devnet().await?;
    Ok(())
}
