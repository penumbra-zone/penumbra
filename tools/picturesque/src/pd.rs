use anyhow::{anyhow, Context as _};
use std::{path::PathBuf, process::Stdio, time::Duration};

use tokio::{fs::create_dir_all, process::Command};

#[derive(Debug, Clone)]
struct Context {
    network_home: PathBuf,
    pd_home: PathBuf,
    log_file: PathBuf,
    log_level: tracing::Level,
}

impl Context {
    fn new(root: PathBuf, log_level: tracing::Level) -> Self {
        Self {
            network_home: root.join("nodes"),
            pd_home: root.join("nodes/node0/pd"),
            log_file: root.join("log/pd.txt"),
            log_level,
        }
    }

    async fn create_directories(&self) -> anyhow::Result<()> {
        create_dir_all(&self.pd_home).await?;
        if let Some(dir) = &self.log_file.parent() {
            create_dir_all(dir).await?;
        }
        Ok(())
    }

    #[tracing::instrument]
    async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("starting pd");
        let file = tokio::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.log_file)
            .await?
            .try_into_std()
            .map_err(|e| anyhow!("{:?}", e))?;
        let mut child = Command::new("pd")
            .env("RUST_LOG", self.log_level.to_string())
            .args([
                "start".as_ref(),
                "--home".as_ref(),
                self.pd_home.as_os_str(),
            ])
            .stdout(Stdio::from(file))
            .stderr(Stdio::null())
            .spawn()?;
        child.wait().await?;
        Ok(())
    }
}

#[tracing::instrument]
pub async fn run(
    root: PathBuf,
    log_level: tracing::Level,
    delay: Option<Duration>,
) -> anyhow::Result<()> {
    if let Some(delay) = delay {
        tracing::debug!(delay = ?delay, "sleeping");
        tokio::time::sleep(delay).await;
    }
    let ctx = Context::new(root, log_level);
    ctx.create_directories().await?;
    ctx.run().await.with_context(|| "while running pd")?;
    Ok(())
}

#[tracing::instrument]
pub async fn generate(
    root: PathBuf,
    log_level: tracing::Level,
    epoch_duration: u32,
) -> anyhow::Result<()> {
    create_dir_all(&root).await?;
    let ctx = Context::new(root, log_level);
    tracing::info!("running pd generate");
    let output = Command::new("pd")
        .args([
            "network".as_ref(),
            "--network-dir".as_ref(),
            ctx.network_home.as_os_str(),
            "generate".as_ref(),
            "--epoch-duration".as_ref(),
            epoch_duration.to_string().as_ref(),
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!(
            "pd network generate returned an error:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
