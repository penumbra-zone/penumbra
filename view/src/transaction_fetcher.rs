use std::collections::BTreeSet;

use anyhow::Context;
use penumbra_crypto::Nullifier;

use penumbra_proto::Protobuf;
use penumbra_tct::Commitment;
use penumbra_transaction::Transaction;
use sha2::Digest;
use tokio::sync::mpsc;

use crate::{sync::FilteredBlock, Storage};

pub struct TransactionFetcher {
    storage: Storage,

    block_rx: mpsc::Receiver<(FilteredBlock, tendermint::Block)>,
}

impl TransactionFetcher {
    pub fn new(
        storage: Storage,
        mut filtered_block_rx: mpsc::Receiver<FilteredBlock>,
        tendermint_url: String,
    ) -> anyhow::Result<TransactionFetcher> {
        let (block_tx, block_rx) = mpsc::channel::<(FilteredBlock, tendermint::Block)>(10);

        use tendermint_rpc::{Client, HttpClient};

        let client = HttpClient::new(tendermint_url.as_str())?;

        tokio::spawn(async move {
            while let Some(filtered_block) = filtered_block_rx.recv().await {
                if filtered_block.height == 0 {
                    // The genesis CompactBlock doesn't correspond to a real block.
                    continue;
                }

                // Only fetch full blocks if we detect spends.
                // TODO: in the future, we could consider chaff downloads.
                if filtered_block.spent_nullifiers.is_empty()
                    && filtered_block.spent_quarantined_nullifiers.is_empty()
                {
                    continue;
                }
                tracing::info!(height = filtered_block.height, "downloading full block");

                match client
                    .block(
                        tendermint::block::Height::try_from(filtered_block.height)
                            .expect("height should be less than 2^63"),
                    )
                    .await
                {
                    Ok(rsp) => {
                        // Pass along the full block.
                        // Send errors indicate the reciever is gone and we should exit.
                        if block_tx.send((filtered_block, rsp.block)).await.is_err() {
                            return;
                        }
                    }
                    Err(e) => {
                        // TODO: how do we handle failures in extended
                        // transaction fetching?  We can't easily resume in
                        // this setup, because the transaction fetcher is
                        // operating on the FilteredBlocks that are produced
                        // as a side effect of syncing, so we can't pick up
                        // from where we left off.

                        // For now: scream loudly and then skip this block.
                        tracing::error!(
                            height = filtered_block.height,
                            ?e,
                            "error trying to fetch extended transaction info"
                        );
                    }
                };
            }
        });

        Ok(TransactionFetcher { storage, block_rx })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        while let Some((filtered_block, block)) = self.block_rx.recv().await {
            // Build list of nullifiers (final or quarantined) from filtered block
            let mut nullifiers: BTreeSet<Nullifier> = filtered_block
                .spent_nullifiers
                .iter()
                .map(|x| x.to_owned())
                .collect();
            for nfs in filtered_block.spent_quarantined_nullifiers.values() {
                nullifiers.extend(nfs);
            }

            for transaction in block.data.iter() {
                // TODO: error handling story?
                let transaction = Transaction::decode(transaction.as_slice())?;

                // Check if each parsed Transaction contains any of the above note commitments or nullifiers

                let mut matched = false;

                for nf in transaction.spent_nullifiers() {
                    if nullifiers.contains(&nf) {
                        matched = true;
                    }
                }

                // If it does contain any of these, insert the Transaction data into the appropriate tables
                if matched {
                    self.storage.record_transaction(transaction).await?;
                }
            }
        }
        Ok(())
    }
}
