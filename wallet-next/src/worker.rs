use std::sync::{Arc, Mutex};

use crate::{sync::scan_block, Storage};
use penumbra_crypto::{merkle::NoteCommitmentTree, FullViewingKey};
use penumbra_proto::client::oblivious::{
    oblivious_query_client::ObliviousQueryClient, CompactBlockRangeRequest,
};
use tonic::transport::Channel;
#[derive(Clone)]
pub struct Worker {
    storage: Storage,
    client: ObliviousQueryClient<Channel>,
    nct: NoteCommitmentTree,
    fvk: FullViewingKey, // TODO: notifications (see TODOs on WalletService)
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
}

impl Worker {
    pub async fn new(
        storage: Storage,
        client: ObliviousQueryClient<Channel>,
        error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    ) -> Result<Self, anyhow::Error> {
        let nct = storage.note_commitment_tree().await?;
        let fvk = storage.full_viewing_key().await?;
        Ok(Self {
            storage,
            client,
            nct,
            fvk,
            error_slot,
        })
    }

    pub async fn sync_to_latest(&mut self) -> Result<u64, anyhow::Error> {
        // Do a single sync run, up to whatever the latest block height is
        tracing::info!("starting client sync");

        let start_height = self
            .storage
            .last_sync_height()
            .await?
            .map(|h| h + 1)
            .unwrap_or(0);

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
            let scan_result = scan_block(&self.fvk, &mut self.nct, block.try_into()?);

            self.storage
                .record_block(scan_result, &mut self.nct)
                .await?;
        }

        let end_height = self.storage.last_sync_height().await?.unwrap();

        tracing::info!(?end_height, "finished sync");

        Ok(end_height)
    }

    pub async fn run(mut self) -> Result<(), anyhow::Error> {
        loop {
            match self._run().await {
                Ok(_) => {}
                Err(e) => {
                    tracing::info!(?e, "wallet worker error");
                    self.error_slot.lock().unwrap().replace(e);
                }
            };
        }
    }

    async fn _run(&mut self) -> Result<(), anyhow::Error> {
        loop {
            self.sync_to_latest().await?;

            // TODO 1: randomize sleep interval within some range?
            // TODO 2: use websockets to be notified on new block
            tokio::time::sleep(std::time::Duration::from_millis(1729)).await;
        }
    }
}
