//! Background worker for continuous compliance scanning.
//!
//! This worker connects to a Penumbra node and continuously scans blocks
//! for compliance-relevant transfers.

use anyhow::{Context, Result};
use penumbra_sdk_keys::keys::MasterComplianceKey;
use penumbra_sdk_proto::core::{
    app::v1::{
        query_service_client::QueryServiceClient as AppQueryServiceClient,
        TransactionsByHeightRequest,
    },
    component::compact_block::v1::{
        query_service_client::QueryServiceClient as CompactBlockQueryServiceClient,
        CompactBlockRangeRequest,
    },
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;
use tonic::transport::Channel;
use tracing::{debug, info, instrument, warn};

use super::storage::ComplianceStorage;
use super::sync::scan_transaction_for_compliance;

// Maximum size of a compact block, in bytes (12MB).
const MAX_CB_SIZE_BYTES: usize = 12 * 1024 * 1024;

/// Background worker that continuously scans blocks for compliance activity.
pub struct ComplianceWorker {
    /// The master compliance key used for scanning.
    mck: MasterComplianceKey,
    /// Storage for persisting detected transfers.
    storage: Arc<ComplianceStorage>,
    /// gRPC channel to the Penumbra node.
    channel: Channel,
    /// Shared error slot for communicating errors to callers.
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    /// Sender for notifying about sync progress.
    sync_height_tx: watch::Sender<u64>,
}

impl ComplianceWorker {
    /// Create a new compliance worker.
    ///
    /// # Arguments
    /// * `mck` - The master compliance key for scanning
    /// * `storage` - Storage backend for persisting results
    /// * `channel` - gRPC channel to the Penumbra node
    ///
    /// # Returns
    /// A tuple containing:
    /// - The worker instance
    /// - An error slot for checking worker health
    /// - A watch receiver for monitoring sync progress
    pub fn new(
        mck: MasterComplianceKey,
        storage: ComplianceStorage,
        channel: Channel,
    ) -> (
        Self,
        Arc<Mutex<Option<anyhow::Error>>>,
        watch::Receiver<u64>,
    ) {
        let storage = Arc::new(storage);
        let error_slot = Arc::new(Mutex::new(None));
        let last_height = storage.last_sync_height().unwrap_or(0);
        let (sync_height_tx, sync_height_rx) = watch::channel(last_height);

        let worker = Self {
            mck,
            storage,
            channel,
            error_slot: error_slot.clone(),
            sync_height_tx,
        };

        (worker, error_slot, sync_height_rx)
    }

    /// Run the worker, continuously syncing blocks.
    ///
    /// This method will run indefinitely, streaming new blocks from the node
    /// and scanning them for compliance activity.
    #[instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        info!("starting compliance scanner worker");

        if let Err(e) = self.sync().await {
            // Store error in error slot
            *self.error_slot.lock().unwrap() = Some(e.context("sync failed"));
            return Err(anyhow::anyhow!("sync failed"));
        }

        Ok(())
    }

    /// Main sync loop - fetches and scans blocks continuously.
    #[instrument(skip(self))]
    async fn sync(&self) -> Result<()> {
        let start_height = self.storage.last_sync_height()? + 1;

        info!(
            start_height,
            "beginning compliance scan from height {}", start_height
        );

        // Connect to CompactBlock service
        let mut compact_block_client = CompactBlockQueryServiceClient::new(self.channel.clone())
            .max_decoding_message_size(MAX_CB_SIZE_BYTES);

        // Stream compact blocks (unbounded = stream forever)
        let mut stream = compact_block_client
            .compact_block_range(CompactBlockRangeRequest {
                start_height,
                end_height: 0,    // 0 = unbounded
                keep_alive: true, // Stream new blocks as they arrive
            })
            .await
            .context("failed to start compact block stream")?
            .into_inner();

        info!("connected to compact block stream");

        // Process blocks from the stream
        while let Some(response) = stream.message().await? {
            let compact_block = response
                .compact_block
                .ok_or_else(|| anyhow::anyhow!("empty compact block"))?;

            let height = compact_block.height;

            // Skip blocks with no data
            if compact_block.state_payloads.is_empty() && compact_block.nullifiers.is_empty() {
                debug!(height, "skipping empty block");
                self.storage.update_sync_height(height)?;
                let _ = self.sync_height_tx.send(height);
                continue;
            }

            // Fetch full transactions for this block
            let transactions = self.fetch_transactions(height).await?;

            if !transactions.is_empty() {
                debug!(height, tx_count = transactions.len(), "scanning block");

                // Compute date from height for key derivation
                let date = self.height_to_date(height);

                // Scan each transaction
                let mut all_transfers = Vec::new();
                for tx in transactions {
                    // Scan for all regulated assets (target_asset_id = None)
                    match scan_transaction_for_compliance(&tx, height, &self.mck, date, None) {
                        Ok(transfers) => {
                            all_transfers.extend(transfers);
                        }
                        Err(e) => {
                            warn!(height, "failed to scan transaction: {}", e);
                        }
                    }
                }

                // Save detected transfers
                if !all_transfers.is_empty() {
                    info!(
                        height,
                        count = all_transfers.len(),
                        "detected compliance transfers"
                    );
                    self.storage.save_transfers(&all_transfers)?;
                }
            }

            // Update sync height
            self.storage.update_sync_height(height)?;
            let _ = self.sync_height_tx.send(height);

            if height % 100 == 0 {
                info!(height, "synced to height {}", height);
            }
        }

        Ok(())
    }

    /// Fetch all transactions for a specific block height.
    #[instrument(skip(self))]
    async fn fetch_transactions(
        &self,
        height: u64,
    ) -> Result<Vec<penumbra_sdk_proto::core::transaction::v1::Transaction>> {
        let mut client = AppQueryServiceClient::new(self.channel.clone());

        let response = client
            .transactions_by_height(TransactionsByHeightRequest {
                block_height: height,
            })
            .await
            .context("failed to fetch transactions")?
            .into_inner();

        Ok(response.transactions)
    }

    /// Convert block height to a date value for scanner key derivation.
    ///
    /// This is a simplified implementation for demo purposes.
    /// In production, this would use the actual block timestamp.
    fn height_to_date(&self, height: u64) -> u64 {
        // Simple formula: assume 1 block per 5 seconds, convert to days
        // 17280 blocks per day (86400 seconds / 5 seconds per block)
        height / 17280
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_height_to_date() {
        let mck = MasterComplianceKey::demo();
        let storage = ComplianceStorage::new(":memory:").unwrap();
        let channel = Channel::from_static("http://localhost:8080").connect_lazy();
        let (worker, _, _) = ComplianceWorker::new(mck, storage, channel);

        // Test date calculation
        assert_eq!(worker.height_to_date(0), 0);
        assert_eq!(worker.height_to_date(17280), 1); // 1 day
        assert_eq!(worker.height_to_date(34560), 2); // 2 days
    }
}
