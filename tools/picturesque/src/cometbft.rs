use anyhow::{anyhow, Context as _};
use std::{fs::create_dir_all, path::PathBuf, process::Stdio, time::Duration};

use tokio::process::Command;

#[derive(Debug, Clone)]
struct Context {
    cometbft_home: PathBuf,
    log_file: PathBuf,
}

impl Context {
    fn new(root: PathBuf) -> Self {
        Self {
            cometbft_home: root.join("nodes/node0/cometbft"),
            log_file: root.join("log/cometbft.txt"),
        }
    }

    fn create_directories(&self) -> anyhow::Result<()> {
        create_dir_all(&self.cometbft_home)?;
        if let Some(dir) = &self.log_file.parent() {
            create_dir_all(dir)?;
        }
        Ok(())
    }

    #[tracing::instrument]
    async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("starting cometbft");
        let file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.log_file)
            .await?
            .try_into_std()
            .map_err(|e| anyhow!("{:?}", e))?;
        let mut child = Command::new("cometbft")
            .args([
                "start".as_ref(),
                "--home".as_ref(),
                self.cometbft_home.as_os_str(),
            ])
            .stdout(Stdio::from(file))
            .stderr(Stdio::null())
            .spawn()?;
        child.wait().await?;
        Ok(())
    }
}

#[tracing::instrument]
pub async fn run(root: PathBuf, delay: Option<Duration>) -> anyhow::Result<()> {
    if let Some(delay) = delay {
        tracing::debug!(delay = ?delay, "sleeping");
        tokio::time::sleep(delay).await;
    }
    let ctx = Context::new(root);
    ctx.create_directories()?;
    ctx.run().await.with_context(|| "while running cometbft")?;
    Ok(())
}
