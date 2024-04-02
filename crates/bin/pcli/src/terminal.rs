use anyhow::Result;
use penumbra_custody::threshold::{SigningRequest, Terminal};
use tokio::io::{self, AsyncBufReadExt};
use tonic::async_trait;

/// For threshold custody, we need to implement this weird terminal abstraction.
///
/// This actually does stuff to stdin and stdout.
pub struct ActualTerminal;

#[async_trait]
impl Terminal for ActualTerminal {
    async fn confirm_request(&self, signing_request: &SigningRequest) -> Result<bool> {
        let (description, json) = match signing_request {
            SigningRequest::TransactionPlan(plan) => {
                ("transaction", serde_json::to_string_pretty(plan)?)
            }
            SigningRequest::ValidatorDefinition(def) => {
                ("validator definition", serde_json::to_string_pretty(def)?)
            }
            SigningRequest::ValidatorVote(vote) => {
                ("validator vote", serde_json::to_string_pretty(vote)?)
            }
        };
        println!("Do you approve this {description}?");
        println!("{json}");
        println!("Press enter to continue");
        self.next_response().await?;
        Ok(true)
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
