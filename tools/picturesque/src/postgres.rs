use anyhow::anyhow;
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::process::Command;
use tokio::{io::AsyncWriteExt as _, task::JoinHandle};

const VENDOR_SQL: &'static str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/util/cometindex/vendor/schema.sql"
));

const DB_NAME: &'static str = "penumbra_raw";

#[derive(Clone, Debug)]
struct Context {
    /// Where postgres should put its data.
    data_dir: PathBuf,
    /// Where postgres should put its communication socket files.
    sock_dir: PathBuf,
    /// Where postgres should put its logs into.
    log_file: PathBuf,
    /// The name of the database.
    db_name: String,
}

impl Context {
    fn new(root: &Path) -> Self {
        let data_dir = root.join("postgres/data");
        let sock_dir = root.join("postgres/sock");
        let log_file = root.join("log/postgres.txt");
        Self {
            data_dir,
            sock_dir,
            log_file,
            db_name: DB_NAME.to_string(),
        }
    }

    fn create_directories(&self) -> anyhow::Result<()> {
        create_dir_all(&self.data_dir)?;
        create_dir_all(&self.sock_dir)?;
        if let Some(dir) = &self.log_file.parent() {
            create_dir_all(dir)?;
        }
        Ok(())
    }

    fn is_postgres_initialized(&self) -> bool {
        // The `initdb` process creates the file `PG_VERSION`, so we'll check for that
        // and skip rerunning the command if found.
        let pg_version = self.data_dir.clone().join("PG_VERSION");
        pg_version.exists()
    }

    #[tracing::instrument]
    async fn init_postgres(&self) -> anyhow::Result<()> {
        if self.is_postgres_initialized() {
            tracing::debug!("postgres data directory already initialized");
            return Ok(());
        }
        let output = Command::new("initdb")
            .args(["-D".as_ref(), self.data_dir.as_os_str()])
            .output()
            .await?;
        if !output.status.success() {
            tracing::error!(
                exit_code = output.status.code(),
                stdout = String::from_utf8_lossy(&output.stdout).to_string(),
                stderr = String::from_utf8_lossy(&output.stderr).to_string(),
                "initdb"
            );
            anyhow::bail!("failed to initialize postgres");
        }
        let mut conf_file = tokio::fs::OpenOptions::new()
            .append(true)
            .open(self.data_dir.join("postgresql.conf"))
            .await?;
        conf_file
            .write_all("listen_addresses=''".as_bytes())
            .await?;
        Ok(())
    }

    async fn run(&self) -> anyhow::Result<()> {
        let (log_dir, log_file_in_dir) = {
            let log_dir = self.log_file.parent().expect("log file should have parent");
            let file_in_dir = self
                .log_file
                .file_name()
                .expect("log file should have file name");
            (log_dir, file_in_dir)
        };
        let log_dir_arg = format!("log_directory={}", log_dir.to_string_lossy());
        let log_filename_arg = format!("log_filename={}", log_file_in_dir.to_string_lossy());
        let args = [
            "-D".as_ref(),
            self.data_dir.as_os_str(),
            "-k".as_ref(),
            self.sock_dir.as_os_str(),
            "-c".as_ref(),
            "logging_collector=on".as_ref(),
            "-c".as_ref(),
            log_dir_arg.as_ref(),
            "-c".as_ref(),
            log_filename_arg.as_ref(),
        ];
        let output = Command::new("postgres").args(args).output().await?;
        if output.status.success() {
            tracing::debug!(
                stdout = String::from_utf8_lossy(&output.stdout).to_string(),
                stderr = String::from_utf8_lossy(&output.stderr).to_string(),
                "postgres early shutdown"
            );
            Ok(())
        } else {
            Err(anyhow!(
                "failed to start postgres\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    async fn database_exists(&self) -> anyhow::Result<bool> {
        let output = Command::new("psql")
            .args([
                "-h".as_ref(),
                self.sock_dir.as_os_str(),
                "-d".as_ref(),
                self.db_name.as_ref(),
                "-lt".as_ref(),
            ])
            .output()
            .await?;
        let exists = output.status.success();
        if !exists {
            tracing::debug!(
                stdout = String::from_utf8_lossy(&output.stdout).to_string(),
                stderr = String::from_utf8_lossy(&output.stderr).to_string(),
                database = &self.db_name,
                "database doesn't exist"
            );
        }
        Ok(exists)
    }

    async fn create_db(&self) -> anyhow::Result<()> {
        let output = Command::new("createdb")
            .args([
                "-h".as_ref(),
                self.sock_dir.as_os_str(),
                self.db_name.as_ref(),
            ])
            .output()
            .await?;
        anyhow::ensure!(
            output.status.success(),
            "failed to create database {}\nstdout: {}\nstderr: {}",
            &self.db_name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    async fn initialize_schema(&self) -> anyhow::Result<()> {
        let mut child = Command::new("psql")
            .args([
                "-h".as_ref(),
                self.sock_dir.as_os_str(),
                "-d".as_ref(),
                self.db_name.as_ref(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child.stdin.take().ok_or(anyhow!("missing stdin"))?;
        stdin.write_all(VENDOR_SQL.as_bytes()).await?;
        let output = child.wait_with_output().await?;
        anyhow::ensure!(
            output.status.success(),
            "failed to initialize database {}\nstdout: {}\nstderr: {}",
            &self.db_name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    #[tracing::instrument]
    async fn init_db(&self) -> anyhow::Result<()> {
        if self.database_exists().await? {
            tracing::info!(database = self.db_name, "already initialized");
            return Ok(());
        }
        self.create_db().await?;
        self.initialize_schema().await?;
        tracing::info!(database = self.db_name, "created and initialized");
        Ok(())
    }

    fn go_connection_string(&self) -> String {
        format!(
            "host={} dbname={} sslmode=disable",
            self.sock_dir.to_string_lossy(),
            self.db_name.as_str()
        )
    }
}

/// Run postgres until it terminates.
///
/// This should be spawned, because running postgres is expected to just continue if no issues arise.
///
/// This will take care of initializing a database with the right scheme if necessary.
#[tracing::instrument]
pub async fn run(root: PathBuf) -> anyhow::Result<()> {
    let ctx = Context::new(&root);
    ctx.create_directories()?;
    ctx.init_postgres().await?;
    let postgres: JoinHandle<anyhow::Result<()>> = {
        let ctx = ctx.clone();
        tokio::spawn(async move { ctx.run().await })
    };
    let init_db: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
        // Give some time for postgres to spin up.
        tokio::time::sleep(Duration::from_millis(1000)).await;
        ctx.init_db().await?;
        Ok(())
    });
    // Try and get the initialization process to complete first.
    init_db.await??;
    // Then, we block this process on postgres.
    postgres.await?
}

/// Given a root directory, return a connection string suitable for Go programs.
///
/// We need this because cometbft will use it.
pub fn go_connection_string(root: &Path) -> String {
    Context::new(root).go_connection_string()
}
