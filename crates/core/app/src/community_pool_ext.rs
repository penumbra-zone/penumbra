use anyhow::Result;
use async_trait::async_trait;

use cnidarium::{StateRead, StateWrite};
use futures::{StreamExt, TryStreamExt};
use penumbra_chain::component::StateReadExt;
use penumbra_governance::state_key;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_transaction::Transaction;

// Note: These should live in `penumbra-governance` in the `StateReadExt` and `StateWriteExt`
// traits, however that would result in a circular dependency since the below methods
// require use of `penumbra-transaction::Transaction`, which has `penumbra-governance` as a
// dependency.

#[async_trait]
pub trait CommunityPoolStateReadExt: StateRead + penumbra_stake::StateReadExt {
    /// Get all the transactions set to be delivered in this block (scheduled in last block).
    async fn pending_community_pool_transactions(&self) -> Result<Vec<Transaction>> {
        // Get the proposal IDs of the Community Pool transactions we are about to deliver.
        let prefix = state_key::deliver_community_pool_transactions_at_height(
            self.get_block_height().await?,
        );
        let proposals: Vec<u64> = self
            .prefix_proto::<u64>(&prefix)
            .map(|result| anyhow::Ok(result?.1))
            .try_collect()
            .await?;

        // For each one, look up the corresponding built transaction, and return the list.
        let mut transactions = Vec::new();
        for proposal in proposals {
            transactions.push(
                self.get(&state_key::community_pool_transaction(proposal))
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("no transaction found for proposal {}", proposal)
                    })?,
            );
        }
        Ok(transactions)
    }
}

impl<T: StateRead + penumbra_stake::StateReadExt + ?Sized> CommunityPoolStateReadExt for T {}

#[async_trait]
pub trait CommunityPoolStateWriteExt: StateWrite {
    /// Get all the transactions set to be delivered in this block (scheduled in last block).
    fn put_community_pool_transaction(&mut self, proposal: u64, transaction: Transaction) {
        self.put(state_key::community_pool_transaction(proposal), transaction);
    }
}

impl<T: StateWrite + ?Sized> CommunityPoolStateWriteExt for T {}
