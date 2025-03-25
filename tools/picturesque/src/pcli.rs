use anyhow::anyhow;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::fs::create_dir_all;
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;

#[derive(Debug, Clone)]
struct Context {
    pcli_home: PathBuf,
}

impl Context {
    fn new(root: PathBuf) -> Self {
        Self {
            pcli_home: root.join("wallets/000"),
        }
    }
}

#[tracing::instrument]
pub async fn init_with_test_keys(root: PathBuf) -> anyhow::Result<()> {
    tracing::info!("running pcli init");
    create_dir_all(&root).await?;
    let ctx = Context::new(root);
    let mut child = Command::new("pcli")
        .args([
            "--home".as_ref(),
            ctx.pcli_home.as_os_str(),
            "init".as_ref(),
            "--grpc-url".as_ref(),
            "http://localhost:8080".as_ref(),
            "soft-kms".as_ref(),
            "import-phrase".as_ref(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().expect("child process should have stdin");
    stdin
        .write_all(penumbra_sdk_keys::test_keys::SEED_PHRASE.as_bytes())
        .await?;
    stdin.write_all("\n".as_bytes()).await?;
    drop(stdin);
    let output = child.wait_with_output().await?;

    if !output.status.success() {
        return Err(anyhow!(
            "pcli init returned an error:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
