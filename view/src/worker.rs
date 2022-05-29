use std::sync::{Arc, Mutex};

use crate::{sync::scan_block, Storage};
use penumbra_crypto::{Asset, FullViewingKey};
use penumbra_proto::client::oblivious::{
    oblivious_query_client::ObliviousQueryClient, AssetListRequest, CompactBlockRangeRequest,
};
use tokio::sync::{watch, RwLock};
use tonic::transport::Channel;
pub struct Worker {
    storage: Storage,
    client: ObliviousQueryClient<Channel>,
    nct: Arc<RwLock<penumbra_tct::Tree>>,
    fvk: FullViewingKey, // TODO: notifications (see TODOs on ViewService)
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    sync_height_tx: watch::Sender<u64>,
}

impl Worker {
    pub async fn new(
        storage: Storage,
        client: ObliviousQueryClient<Channel>,
        error_slot: Arc<Mutex<Option<anyhow::Error>>>,
        sync_height_tx: watch::Sender<u64>,
    ) -> Result<(Self, Arc<RwLock<penumbra_tct::Tree>>), anyhow::Error> {
        let nct = Arc::new(RwLock::new(storage.note_commitment_tree().await?));
        let fvk = storage.full_viewing_key().await?;
        Ok((
            Self {
                storage,
                client,
                nct: nct.clone(),
                fvk,
                error_slot,
                sync_height_tx,
            },
            nct,
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

    pub async fn sync_to_latest(&mut self) -> Result<u64, anyhow::Error> {
        // Do a single sync run, up to whatever the latest block height is
        tracing::info!("starting client sync");

        // Lock the NCT during sync
        let mut nct = self.nct.write().await;

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
                start_height,
                end_height: 0,
                chain_id: self.storage.chain_params().await?.chain_id,
            }))
            .await?
            .into_inner();

        while let Some(block) = stream.message().await? {
            let scan_result = scan_block(&self.fvk, &mut nct, block.try_into()?, epoch_duration);
            let height = scan_result.height;

            self.storage.record_block(scan_result, &mut nct).await?;
            // Notify all watchers of the new height we just recorded.
            self.sync_height_tx.send(height)?;
        }

        let end_height = self.storage.last_sync_height().await?.unwrap();

        // Release the NCT RwLock
        drop(nct);

        tracing::info!(?end_height, "finished sync");

        Ok(end_height)
    }

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
        loop {
            self.sync_to_latest().await?;

            if self.sync_height_tx.is_closed() {
                tracing::info!("all view services dropped, shutting down worker");
                break;
            }

            // TODO 1: randomize sleep interval within some range?
            // TODO 2: use websockets to be notified on new block
            tokio::time::sleep(std::time::Duration::from_millis(1729)).await;
        }

        // If this is returned, it means the loop was broken by a shutdown signal.
        Ok(())
    }
}
