use anyhow::Result;
use penumbra_custody::threshold::Terminal;
use penumbra_transaction::plan::TransactionPlan;
use tonic::async_trait;

/// For threshold custody, we need to implement this weird terminal abstraction.
///
/// This actually does stuff to stdin and stdout.
pub struct ActualTerminal;

#[async_trait]
impl Terminal for ActualTerminal {
    async fn confirm_transaction(&self, _transaction: &TransactionPlan) -> Result<bool> {
        todo!()
    }

    async fn explain(&self, _msg: &str) -> Result<()> {
        todo!()
    }

    async fn broadcast(&self, _data: &str) -> Result<()> {
        todo!()
    }

    async fn next_response(&self) -> Result<Option<String>> {
        todo!()
    }
}
