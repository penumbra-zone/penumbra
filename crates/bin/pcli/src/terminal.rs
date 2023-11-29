use anyhow::Result;
use penumbra_custody::threshold::Terminal;
use penumbra_transaction::plan::TransactionPlan;
use tokio::io::{self, AsyncBufReadExt};
use tonic::async_trait;

/// For threshold custody, we need to implement this weird terminal abstraction.
///
/// This actually does stuff to stdin and stdout.
pub struct ActualTerminal;

#[async_trait]
impl Terminal for ActualTerminal {
    async fn confirm_transaction(&self, transaction: &TransactionPlan) -> Result<bool> {
        println!("Do you approve this transaction?");
        println!("{}", serde_json::to_string_pretty(transaction)?);
        println!("Y/N?");
        let response = self.next_response().await?;
        Ok(response.map(|x| x.to_lowercase() == "y").unwrap_or(false))
    }

    async fn explain(&self, msg: &str) -> Result<()> {
        println!("{}", msg);
        Ok(())
    }

    async fn broadcast(&self, data: &str) -> Result<()> {
        println!("{}", data);
        Ok(())
    }

    async fn next_response(&self) -> Result<Option<String>> {
        let stdin = io::stdin();
        let mut stdin = io::BufReader::new(stdin);

        let mut line = String::new();
        stdin.read_line(&mut line).await?;

        if line.is_empty() {
            return Ok(None);
        }
        Ok(Some(line))
    }
}
