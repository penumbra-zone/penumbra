use std::sync::{Arc, Mutex};

use crate::{sync::scan_block, Storage};
use penumbra_chain::sync::CompactBlock;
use penumbra_crypto::{Asset, FullViewingKey};
use penumbra_proto::client::oblivious::{
    oblivious_query_client::ObliviousQueryClient, AssetListRequest, CompactBlockRangeRequest,
};
#[cfg(feature = "nct-divergence-check")]
use penumbra_proto::client::specific::specific_query_client::SpecificQueryClient;
use tokio::sync::{watch, RwLock};
use tonic::transport::Channel;
pub struct Worker {
    storage: Storage,
    client: ObliviousQueryClient<Channel>,
    nct: Arc<RwLock<penumbra_tct::Tree>>,
    fvk: FullViewingKey, // TODO: notifications (see TODOs on ViewService)
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    sync_height_tx: watch::Sender<u64>,
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

        Ok((
            Self {
                storage,
                client,
                nct: nct.clone(),
                fvk,
                error_slot: error_slot.clone(),
                sync_height_tx,
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

        use std::collections::BTreeSet;
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

        while let Some(block) = stream.message().await? {
            let block = CompactBlock::try_from(block)?;
            let height = block.height;

            // Lock the NCT only while processing this block.
            let mut nct_guard = self.nct.write().await;

            if block.is_empty() {
                // Optimization: if the block is empty, seal the in-memory NCT,
                // and skip touching the database:
                nct_guard.end_block().unwrap();
                self.storage.record_empty_block(height).await?;
                // Notify all watchers of the new height we just recorded.
                self.sync_height_tx.send(height)?;
            } else {
                // Otherwise, scan the block and commit its changes:
                let scan_result =
                    scan_block(&self.fvk, &mut nct_guard, block.try_into()?, epoch_duration);

                self.storage
                    .record_block(scan_result, &mut nct_guard)
                    .await?;
                // Notify all watchers of the new height we just recorded.
                self.sync_height_tx.send(height)?;
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

    //TODO: should this actually be looping? seems worth revisiting, because right now it either breaks or errors once.
    #[allow(clippy::never_loop)]
    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        loop {
            match self.run_inner().await {
                Ok(_) => {
                    // If the worker returns `Ok` then it means it's done, so we can
                    // stop looping.
                    break;
                }
                Err(e) => {
                    tracing::info!(?e, "view worker error");
                    self.error_slot.lock().unwrap().replace(e);
                    // Exit the worker to avoid looping endlessly.
                    return Err(anyhow::anyhow!("view worker error"));
                }
            };
        }

        Ok(())
    }

    async fn run_inner(&mut self) -> Result<(), anyhow::Error> {
        // For now, this can be outside of the loop, because assets are only
        // created at genesis. In the future, we'll want to have a way for
        // clients to learn about assets as they're created.
        self.fetch_assets().await?;

        let mut error_count = 0;
        loop {
            match self.sync().await {
                // If the sync returns `Ok` then it means we're shutting down.
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::warn!(?e);
                    error_count += 1;
                    // Retry a few times and then give up.
                    if error_count > 3 {
                        return Err(e);
                    }
                }
            }
            // Wait a bit before restarting
            tokio::time::sleep(std::time::Duration::from_millis(1729)).await;
        }
    }
}

#[cfg(feature = "nct-divergence-check")]
async fn nct_divergence_check(
    client: &mut SpecificQueryClient<Channel>,
    height: u64,
    actual_root: penumbra_tct::Root,
) -> anyhow::Result<()> {
    use penumbra_proto::Protobuf;

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
