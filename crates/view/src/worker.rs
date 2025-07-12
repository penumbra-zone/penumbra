use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Context;
use penumbra_sdk_auction::auction::AuctionNft;
use penumbra_sdk_compact_block::CompactBlock;
use penumbra_sdk_dex::lp::{position, LpNft};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_proto::core::{
    app::v1::{
        query_service_client::QueryServiceClient as AppQueryServiceClient,
        TransactionsByHeightRequest,
    },
    component::{
        compact_block::v1::{
            query_service_client::QueryServiceClient as CompactBlockQueryServiceClient,
            CompactBlockRangeRequest,
        },
        shielded_pool::v1::{
            query_service_client::QueryServiceClient as ShieldedPoolQueryServiceClient,
            AssetMetadataByIdRequest,
        },
    },
};
use penumbra_sdk_sct::{CommitmentSource, Nullifier};
use penumbra_sdk_transaction::Transaction;
use tap::Tap;
use tokio::sync::{watch, RwLock};
use tonic::transport::Channel;
use tracing::instrument;

use crate::{
    sync::{scan_block, FilteredBlock},
    Storage,
};

// The maximum size of a compact block, in bytes (12MB).
const MAX_CB_SIZE_BYTES: usize = 12 * 1024 * 1024;

pub struct Worker {
    storage: Storage,
    sct: Arc<RwLock<penumbra_sdk_tct::Tree>>,
    fvk: FullViewingKey, // TODO: notifications (see TODOs on ViewService)
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    sync_height_tx: watch::Sender<u64>,
    /// Tonic channel used to create GRPC clients.
    channel: Channel,
}

impl Worker {
    /// Creates a new worker, returning:
    ///
    /// - the worker itself;
    /// - a shared, in-memory SCT instance;
    /// - a shared error slot;
    /// - a channel for notifying the client of sync progress.
    #[instrument(skip_all)]
    pub async fn new(
        storage: Storage,
        channel: Channel,
    ) -> Result<
        (
            Self,
            Arc<RwLock<penumbra_sdk_tct::Tree>>,
            Arc<Mutex<Option<anyhow::Error>>>,
            watch::Receiver<u64>,
        ),
        anyhow::Error,
    > {
        tracing::trace!("constructing view server worker");
        let fvk = storage
            .full_viewing_key()
            .await
            .context("failed to retrieve full viewing key from storage")?
            .tap(|_| tracing::debug!("retrieved full viewing key"));

        // Create a shared, in-memory SCT.
        let sct = Arc::new(RwLock::new(storage.state_commitment_tree().await?));
        // Create a shared error slot
        let error_slot = Arc::new(Mutex::new(None));
        // Create a channel for the worker to notify of sync height changes.
        let (sync_height_tx, mut sync_height_rx) =
            watch::channel(storage.last_sync_height().await?.unwrap_or(0));
        // Mark the current height as seen, since it's not new.
        sync_height_rx.borrow_and_update();

        Ok((
            Self {
                storage,
                sct: sct.clone(),
                fvk,
                error_slot: error_slot.clone(),
                sync_height_tx,
                channel,
            },
            sct,
            error_slot,
            sync_height_rx,
        ))
    }

    pub async fn fetch_transactions(
        &self,
        filtered_block: &mut FilteredBlock,
    ) -> anyhow::Result<Vec<Transaction>> {
        let spent_nullifiers = filtered_block
            .spent_nullifiers
            .iter()
            .cloned()
            .collect::<BTreeSet<Nullifier>>();

        let has_tx_sources = filtered_block
            .new_notes
            .values()
            .map(|record| &record.source)
            .chain(
                filtered_block
                    .new_swaps
                    .values()
                    .map(|record| &record.source),
            )
            .any(|source| matches!(source, CommitmentSource::Transaction { .. }));

        // Only make a block request if we detected transactions in the FilteredBlock.
        // TODO: in the future, we could perform chaff downloads.
        if spent_nullifiers.is_empty() && !has_tx_sources {
            return Ok(Vec::new());
        }

        tracing::debug!(
            height = filtered_block.height,
            "fetching full transaction data"
        );

        let all_transactions =
            fetch_transactions(self.channel.clone(), filtered_block.height).await?;

        let mut transactions = Vec::new();

        for tx in all_transactions {
            let tx_id = tx.id().0;

            let mut relevant = false;

            if tx
                .spent_nullifiers()
                .any(|nf| spent_nullifiers.contains(&nf))
            {
                // The transaction is relevant, it spends one of our nullifiers.
                relevant = true;
            }

            // Rehydrate commitment sources.
            for commitment in tx.state_commitments() {
                filtered_block
                    .new_notes
                    .entry(commitment)
                    .and_modify(|record| {
                        relevant = true;
                        record.source = CommitmentSource::Transaction { id: Some(tx_id) };
                    });
                filtered_block
                    .new_swaps
                    .entry(commitment)
                    .and_modify(|record| {
                        relevant = true;
                        record.source = CommitmentSource::Transaction { id: Some(tx_id) };
                    });
            }

            if relevant {
                transactions.push(tx);
            }
        }

        tracing::debug!(
            matched = transactions.len(),
            "filtered relevant transactions"
        );

        Ok(transactions)
    }

    pub async fn sync(&mut self) -> anyhow::Result<()> {
        // Do a single sync run, up to whatever the latest block height is
        tracing::info!("starting client sync");

        let start_height = self
            .storage
            .last_sync_height()
            .await?
            .map(|h| h + 1)
            .unwrap_or(0);

        let mut client = CompactBlockQueryServiceClient::new(self.channel.clone())
            .max_decoding_message_size(MAX_CB_SIZE_BYTES);
        let mut stream = client
            .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
                start_height,
                end_height: 0,
                // Instruct the server to keep feeding us blocks as they're created.
                keep_alive: true,
            }))
            .await?
            .into_inner();

        // Spawn a task to consume items from the stream (somewhat)
        // independently of the execution of the block scanning.  This has two
        // purposes: first, it allows buffering to smooth performance; second,
        // it makes it slightly more difficult for a remote server to observe
        // the exact timings of the scanning of each CompactBlock.
        let (tx, mut buffered_stream) = tokio::sync::mpsc::channel(1000);
        tokio::spawn(async move {
            while let Some(block) = stream.message().await.transpose() {
                if tx.send(block).await.is_err() {
                    break;
                }
            }
        });

        let mut expected_height = start_height;

        while let Some(block) = buffered_stream.recv().await {
            let block: CompactBlock = block?.try_into()?;

            let height = block.height;
            if height != expected_height {
                tracing::warn!("out of order block detected");
                continue;
            }
            expected_height += 1;

            // Lock the SCT only while processing this block.
            let mut sct_guard = self.sct.write().await;

            if let Some(root) = block.epoch_root {
                // We now know the root for this epoch.
                self.storage
                    .update_epoch(block.epoch_index, Some(root), None)
                    .await?;
                // And also where the next epoch starts, since this block is the last.
                self.storage
                    .update_epoch(block.epoch_index + 1, None, Some(block.height + 1))
                    .await?;
            }

            if !block.requires_scanning() {
                // Optimization: if the block is empty, seal the in-memory SCT,
                // and skip touching the database:
                sct_guard.end_block()?;
                // We also need to end the epoch, since if there are no funding streams, then an
                // epoch boundary won't necessarily require scanning:
                if block.epoch_root.is_some() {
                    sct_guard
                        .end_epoch()
                        .expect("ending the epoch must succeed");
                }
                self.storage.record_empty_block(height).await?;
                // Notify all watchers of the new height we just recorded.
                self.sync_height_tx.send(height)?;
            } else {
                // Otherwise, scan the block and commit its changes:
                let mut filtered_block =
                    scan_block(&self.fvk, &mut sct_guard, block, &self.storage).await?;

                // Download any transactions we detected.
                let transactions = self.fetch_transactions(&mut filtered_block).await?;

                // LPNFT asset IDs won't be known to the chain, so we need to pre-populate them in the local
                // registry based on transaction contents.
                for transaction in &transactions {
                    for action in transaction.actions() {
                        match action {
                            penumbra_sdk_transaction::Action::PositionOpen(position_open) => {
                                let position_id = position_open.position.id();

                                // Record every possible permutation.
                                let lp_nft = LpNft::new(position_id, position::State::Opened);
                                let _id = lp_nft.asset_id();
                                let denom = lp_nft.denom();
                                self.storage.record_asset(denom).await?;

                                let lp_nft = LpNft::new(position_id, position::State::Closed);
                                let _id = lp_nft.asset_id();
                                let denom = lp_nft.denom();
                                self.storage.record_asset(denom).await?;

                                let lp_nft = LpNft::new(
                                    position_id,
                                    position::State::Withdrawn { sequence: 0 },
                                );
                                let _id = lp_nft.asset_id();
                                let denom = lp_nft.denom();
                                self.storage.record_asset(denom).await?;

                                // Record the position itself
                                self.storage
                                    .record_position(position_open.position.clone())
                                    .await?;
                            }
                            penumbra_sdk_transaction::Action::PositionClose(position_close) => {
                                let position_id = position_close.position_id;

                                // Update the position record
                                self.storage
                                    .update_position(position_id, position::State::Closed)
                                    .await?;
                            }
                            penumbra_sdk_transaction::Action::PositionWithdraw(
                                position_withdraw,
                            ) => {
                                let position_id = position_withdraw.position_id;

                                // Record the LPNFT for the current sequence number.
                                let state = position::State::Withdrawn {
                                    sequence: position_withdraw.sequence,
                                };
                                let lp_nft = LpNft::new(position_id, state);
                                let denom = lp_nft.denom();
                                self.storage.record_asset(denom).await?;

                                // Update the position record
                                self.storage.update_position(position_id, state).await?;
                            }
                            penumbra_sdk_transaction::Action::ActionDutchAuctionSchedule(
                                schedule_da,
                            ) => {
                                let auction_id = schedule_da.description.id();
                                let auction_nft_opened = AuctionNft::new(auction_id, 0);
                                let nft_metadata_opened = auction_nft_opened.metadata.clone();

                                self.storage.record_asset(nft_metadata_opened).await?;

                                self.storage
                                    .record_auction_with_state(
                                        schedule_da.description.id(),
                                        0u64, // Opened
                                    )
                                    .await?;
                            }
                            penumbra_sdk_transaction::Action::ActionDutchAuctionEnd(end_da) => {
                                let auction_id = end_da.auction_id;
                                let auction_nft_closed = AuctionNft::new(auction_id, 1);
                                let nft_metadata_closed = auction_nft_closed.metadata.clone();

                                self.storage.record_asset(nft_metadata_closed).await?;

                                self.storage
                                    .record_auction_with_state(end_da.auction_id, 1)
                                    .await?;
                            }
                            penumbra_sdk_transaction::Action::ActionDutchAuctionWithdraw(
                                withdraw_da,
                            ) => {
                                let auction_id = withdraw_da.auction_id;
                                let auction_nft_withdrawn =
                                    AuctionNft::new(auction_id, withdraw_da.seq);
                                let nft_metadata_withdrawn = auction_nft_withdrawn.metadata.clone();

                                self.storage.record_asset(nft_metadata_withdrawn).await?;
                                self.storage
                                    .record_auction_with_state(auction_id, withdraw_da.seq)
                                    .await?;
                            }
                            _ => (),
                        };
                    }
                }

                // Record any new assets we detected.
                for note_record in filtered_block.new_notes.values() {
                    // If the asset is already known, skip it, unless there's useful information
                    // to cross-reference.
                    if let Some(note_denom) = self
                        .storage
                        .asset_by_id(&note_record.note.asset_id())
                        .await?
                    {
                        // If the asset metata is for an auction, we record the associated note commitment
                        // in the auction state table to cross reference with SNRs.
                        if note_denom.is_auction_nft() {
                            let note_commitment = note_record.note_commitment;
                            let auction_nft: AuctionNft = note_denom.try_into()?;
                            self.storage
                                .update_auction_with_note_commitment(
                                    auction_nft.id,
                                    note_commitment,
                                )
                                .await?;
                        } else if let Ok(lp_nft) = LpNft::try_from(note_denom) {
                            self.storage
                                .update_position_with_account(
                                    lp_nft.position_id(),
                                    note_record.address_index.account,
                                )
                                .await?;
                        }
                        continue;
                    } else {
                        // If the asset is unknown, we may be able to query for its denom metadata and store that.

                        let mut client = ShieldedPoolQueryServiceClient::new(self.channel.clone());
                        if let Some(denom_metadata) = client
                            .asset_metadata_by_id(AssetMetadataByIdRequest {
                                asset_id: Some(note_record.note.asset_id().into()),
                            })
                            .await?
                            .into_inner()
                            .denom_metadata
                        {
                            // If we get metadata: great, record it.
                            self.storage
                                .record_asset(denom_metadata.try_into()?)
                                .await?;
                        } else {
                            tracing::warn!(asset_id = ?note_record.note.asset_id(), "received unknown asset ID with no available metadata");
                        }
                    }
                }

                // Commit the block to the database.
                self.storage
                    .record_block(
                        filtered_block.clone(),
                        transactions,
                        &mut sct_guard,
                        self.channel.clone(),
                    )
                    .await?;
                // Notify all watchers of the new height we just recorded.
                self.sync_height_tx.send(filtered_block.height)?;
            }
            #[cfg(feature = "sct-divergence-check")]
            sct_divergence_check(self.channel.clone(), height, sct_guard.root()).await?;

            // Release the SCT RwLock
            drop(sct_guard);

            // Check if we should stop waiting for blocks to arrive, because the view
            // services are dropped and we're supposed to shut down.
            if self.sync_height_tx.is_closed() {
                return Ok(());
            }
        }

        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            // Do a single sync run, recording any errors.
            if let Err(e) = self.sync().await {
                tracing::error!(?e, "view worker error");
                self.error_slot
                    .lock()
                    .expect("mutex is not poisoned")
                    .replace(e);
            }
            // Sleep 10s (maybe later use exponential backoff?)
            tokio::time::sleep(Duration::from_secs(10)).await;
            // Clear the error slot before retrying.
            *self.error_slot.lock().expect("mutex is not poisoned") = None;
        }
    }
}

// Fetches all transactions in the block.
async fn fetch_transactions(
    channel: Channel,
    block_height: u64,
) -> anyhow::Result<Vec<Transaction>> {
    let mut client = AppQueryServiceClient::new(channel);
    let request = TransactionsByHeightRequest {
        block_height,
        ..Default::default()
    };
    // HACK: this is not a robust long-term solution but may help
    // avoid "split-brain" block fetch issues, where a client learns
    // of a new block, then immediately tries to fetch it, but that
    // fetch is load-balanced over a different node that hasn't yet
    // learned about that block.
    let response = match client.transactions_by_height(request.clone()).await {
        Ok(rsp) => rsp,
        Err(e) => {
            tracing::warn!(?e, "failed to fetch block, waiting and retrying once");
            tokio::time::sleep(Duration::from_secs(1)).await;
            client.transactions_by_height(request).await?
        }
    };
    let transactions = response
        .into_inner()
        .transactions
        .into_iter()
        .map(TryInto::try_into)
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(transactions)
}

#[cfg(feature = "sct-divergence-check")]
async fn sct_divergence_check(
    channel: Channel,
    height: u64,
    actual_root: penumbra_sdk_tct::Root,
) -> anyhow::Result<()> {
    use cnidarium::proto::v1::query_service_client::QueryServiceClient;
    use penumbra_sdk_proto::DomainType;
    use penumbra_sdk_sct::state_key as sct_state_key;

    let mut client = QueryServiceClient::new(channel);
    tracing::info!(?height, "fetching anchor @ height");

    let value = client
        .key_value(cnidarium::proto::v1::KeyValueRequest {
            key: sct_state_key::tree::anchor_by_height(height),
            proof: false,
            ..Default::default()
        })
        .await?
        .into_inner()
        .value
        .context("sct state not found")?;

    let expected_root = penumbra_sdk_tct::Root::decode(value.value.as_slice())?;

    if actual_root == expected_root {
        tracing::info!(?height, ?actual_root, ?expected_root, "sct roots match");
        Ok(())
    } else {
        let e = anyhow::anyhow!(
            "SCT divergence detected at height {}: expected {}, got {}",
            height,
            expected_root,
            actual_root
        );
        // Print the error immediately, so that it's visible in the logs.
        tracing::error!(?e);
        Err(e)
    }
}
