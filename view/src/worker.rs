use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use penumbra_chain::{sync::CompactBlock, Epoch};
use penumbra_crypto::{Asset, FullViewingKey, Nullifier};
use penumbra_proto::{
    client::v1alpha1::{
        oblivious_query_client::ObliviousQueryClient, AssetListRequest, CompactBlockRangeRequest,
    },
    Protobuf,
};
use penumbra_transaction::Transaction;
use sha2::Digest;
use tendermint_rpc::Client;
use tokio::sync::{watch, RwLock};
use tonic::transport::Channel;

#[cfg(feature = "nct-divergence-check")]
use penumbra_proto::client::specific::specific_query_client::SpecificQueryClient;

use crate::{
    sync::{scan_block, FilteredBlock},
    Storage,
};

pub struct Worker {
    storage: Storage,
    client: ObliviousQueryClient<Channel>,
    nct: Arc<RwLock<penumbra_tct::Tree>>,
    fvk: FullViewingKey, // TODO: notifications (see TODOs on ViewService)
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    sync_height_tx: watch::Sender<u64>,
    tm_client: tendermint_rpc::HttpClient,
    #[cfg(feature = "nct-divergence-check")]
    specific_client: SpecificQueryClient<Channel>,
}

impl Worker {
    /// Creates a new worker, returning:
    ///
    /// - the worker itself;
    /// - a shared, in-memory NCT instance;
    /// - a shared error slot;
    /// - a channel for notifying the client of sync progress.
    pub async fn new(
        storage: Storage,
        node: String,
        pd_port: u16,
        tendermint_port: u16,
    ) -> Result<
        (
            Self,
            Arc<RwLock<penumbra_tct::Tree>>,
            Arc<Mutex<Option<anyhow::Error>>>,
            watch::Receiver<u64>,
        ),
        anyhow::Error,
    > {
        let fvk = storage.full_viewing_key().await?;

        // Create a shared, in-memory NCT.
        let nct = Arc::new(RwLock::new(storage.note_commitment_tree().await?));
        // Create a shared error slot
        let error_slot = Arc::new(Mutex::new(None));
        // Create a channel for the worker to notify of sync height changes.
        let (sync_height_tx, mut sync_height_rx) =
            watch::channel(storage.last_sync_height().await?.unwrap_or(0));
        // Mark the current height as seen, since it's not new.
        sync_height_rx.borrow_and_update();

        let client = ObliviousQueryClient::connect(format!("http://{}:{}", node, pd_port)).await?;
        #[cfg(feature = "nct-divergence-check")]
        let specific_client =
            SpecificQueryClient::connect(format!("http://{}:{}", node, pd_port)).await?;

        let tm_client = tendermint_rpc::HttpClient::new(
            format!("http://{}:{}", node, tendermint_port).as_str(),
        )?;

        Ok((
            Self {
                storage,
                client,
                nct: nct.clone(),
                fvk,
                error_slot: error_slot.clone(),
                sync_height_tx,
                tm_client,
                #[cfg(feature = "nct-divergence-check")]
                specific_client,
            },
            nct,
            error_slot,
            sync_height_rx,
        ))
    }

    pub async fn fetch_assets(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("fetching assets");

        let chain_id = self.storage.chain_params().await?.chain_id;

        // Hack to work around SQL query -- if we insert duplicate assets with
        // the query, it will give a duplicate key error, so just manually load
        // them all into memory.  better -- fix the sql query

        let known_assets = self
            .storage
            .assets()
            .await?
            .into_iter()
            .map(|asset| asset.id)
            .collect::<BTreeSet<_>>();

        let assets = self
            .client
            .asset_list(tonic::Request::new(AssetListRequest { chain_id }))
            .await?
            .into_inner()
            .assets;

        for new_asset in assets {
            let new_asset = Asset::try_from(new_asset)?;
            if !known_assets.contains(&new_asset.id) {
                self.storage.record_asset(new_asset).await?;
            }
        }

        tracing::info!("updated asset cache");

        Ok(())
    }

    pub async fn fetch_transactions(
        &self,
        filtered_block: &FilteredBlock,
    ) -> anyhow::Result<Vec<Transaction>> {
        let inbound_transaction_ids = filtered_block.inbound_transaction_ids();
        let spent_nullifiers = filtered_block
            .all_nullifiers()
            .cloned()
            .collect::<BTreeSet<Nullifier>>();

        // Only make a block request if we detected transactions in the FilteredBlock.
        // TODO: in the future, we could perform chaff downloads.
        if spent_nullifiers.is_empty() && inbound_transaction_ids.is_empty() {
            return Ok(Vec::new());
        }

        tracing::debug!(
            height = filtered_block.height,
            "fetching full transaction data"
        );

        let block = self
            .tm_client
            .block(
                tendermint::block::Height::try_from(filtered_block.height)
                    .expect("height should be less than 2^63"),
            )
            .await?
            .block;

        let mut transactions = Vec::new();

        for tx_bytes in block.data.iter() {
            let tx_id: [u8; 32] = sha2::Sha256::digest(tx_bytes.as_slice())
                .as_slice()
                .try_into()
                .unwrap();

            let transaction = Transaction::decode(tx_bytes.as_slice())?;

            // Check if the transaction is a known inbound transaction or spends one of our nullifiers.
            if inbound_transaction_ids.contains(&tx_id)
                || transaction
                    .spent_nullifiers()
                    .any(|nf| spent_nullifiers.contains(&nf))
            {
                transactions.push(transaction)
            }
        }
        tracing::debug!(
            transactions_in_block = block.data.len(),
            matched = transactions.len(),
            "filtered relevant transactions"
        );

        Ok(transactions)
    }

    pub async fn sync(&mut self) -> Result<(), anyhow::Error> {
        // Do a single sync run, up to whatever the latest block height is
        tracing::info!("starting client sync");

        let start_height = self
            .storage
            .last_sync_height()
            .await?
            .map(|h| h + 1)
            .unwrap_or(0);

        let epoch_duration = self.storage.chain_params().await?.epoch_duration;

        let mut stream = self
            .client
            .compact_block_range(tonic::Request::new(CompactBlockRangeRequest {
                chain_id: self.storage.chain_params().await?.chain_id,
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

        while let Some(block) = buffered_stream.recv().await {
            let block = CompactBlock::try_from(block?)?;
            let height = block.height;

            // Lock the NCT only while processing this block.
            let mut nct_guard = self.nct.write().await;

            if !block.requires_scanning() {
                // Optimization: if the block is empty, seal the in-memory NCT,
                // and skip touching the database:
                nct_guard.end_block().unwrap();
                // We also need to end the epoch, since if there are no funding streams, then an
                // epoch boundary won't necessarily require scanning:
                if Epoch::from_height(height, epoch_duration).is_epoch_end(height) {
                    nct_guard
                        .end_epoch()
                        .expect("ending the epoch must succeed");
                }
                self.storage.record_empty_block(height).await?;
                // Notify all watchers of the new height we just recorded.
                self.sync_height_tx.send(height)?;
            } else {
                // Otherwise, scan the block and commit its changes:
                let filtered_block = scan_block(
                    &self.fvk,
                    &mut nct_guard,
                    block,
                    epoch_duration,
                    &self.storage,
                )
                .await?;

                // Download any transactions we detected.
                let transactions = self.fetch_transactions(&filtered_block).await?;

                self.storage
                    .record_block(filtered_block.clone(), transactions, &mut nct_guard)
                    .await?;
                // Notify all watchers of the new height we just recorded.
                self.sync_height_tx.send(filtered_block.height)?;
            }
            #[cfg(feature = "nct-divergence-check")]
            nct_divergence_check(&mut self.specific_client, height, nct_guard.root()).await?;

            // Release the NCT RwLock
            drop(nct_guard);

            // Check if we should stop waiting for blocks to arrive, because the view
            // services are dropped and we're supposed to shut down.
            if self.sync_height_tx.is_closed() {
                return Ok(());
            }
        }

        Ok(())
    }

    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        self.run_inner().await.map_err(|e| {
            tracing::info!(?e, "view worker error");
            self.error_slot.lock().unwrap().replace(e);
            anyhow::anyhow!("view worker error")
        })
    }

    async fn run_inner(&mut self) -> Result<(), anyhow::Error> {
        // For now, this can be outside of the loop, because assets are only
        // created at genesis. In the future, we'll want to have a way for
        // clients to learn about assets as they're created.
        self.fetch_assets().await?;
        self.sync().await?;
        Ok(())
    }
}

#[cfg(feature = "nct-divergence-check")]
async fn nct_divergence_check(
    client: &mut SpecificQueryClient<Channel>,
    height: u64,
    actual_root: penumbra_tct::Root,
) -> anyhow::Result<()> {
    let value = client
        .key_value(penumbra_proto::client::specific::KeyValueRequest {
            key: format!("shielded_pool/anchor/{}", height).into_bytes(),
            ..Default::default()
        })
        .await?
        .into_inner()
        .value;

    let expected_root = penumbra_tct::Root::decode(value.as_slice())?;

    if actual_root == expected_root {
        tracing::info!(?height, ?actual_root, ?expected_root, "nct roots match");
        Ok(())
    } else {
        let e = anyhow::anyhow!(
            "NCT divergence detected at height {}: expected {}, got {}",
            height,
            expected_root,
            actual_root
        );
        // Print the error immediately, so that it's visible in the logs.
        tracing::error!(?e);
        Err(e)
    }
}
