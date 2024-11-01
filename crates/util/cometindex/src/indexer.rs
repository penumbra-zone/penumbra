mod indexing_state;

use crate::{index::EventBatch, opt::Options, AppView};
use anyhow::{Context as _, Result};
use indexing_state::{Height, IndexingState};
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinSet};

#[tracing::instrument(skip_all)]
async fn catchup(
    state: &IndexingState,
    indices: &[Arc<dyn AppView>],
    genesis: Arc<serde_json::Value>,
) -> anyhow::Result<()> {
    if indices.len() <= 0 {
        tracing::info!(why = "no indices", "catchup completed");
        return Ok(());
    }

    let (src_height, index_heights) = tokio::try_join!(state.src_height(), state.index_heights())?;
    tracing::info!(?src_height, ?index_heights, "catchup status");
    let lowest_index_height = index_heights.values().copied().min().unwrap_or_default();
    if lowest_index_height >= src_height {
        tracing::info!(why = "already caught up", "catchup completed");
        return Ok(());
    }

    // Constants that influence performance.
    const DEFAULT_BATCH_SIZE: u64 = 1000;
    const BATCH_LOOKAHEAD: usize = 2;

    let mut tasks = JoinSet::<anyhow::Result<()>>::new();

    let mut txs = Vec::with_capacity(indices.len());
    for index in indices.iter().cloned() {
        let (tx, mut rx) = mpsc::channel::<EventBatch>(BATCH_LOOKAHEAD);
        txs.push(tx);
        let name = index.name();
        let index_height = index_heights.get(&name).copied().unwrap_or_default();
        let state_cp = state.clone();
        let genesis_cp = genesis.clone();
        tasks.spawn(async move {
            if index_height == Height::default() {
                tracing::info!(?name, "initializing index");
                let mut dbtx = state_cp.begin_transaction().await?;
                index.init_chain(&mut dbtx, &genesis_cp).await?;
                tracing::info!(?name, "finished initialization");
                IndexingState::update_index_height(&mut dbtx, &name, Height::post_genesis())
                    .await?;
                dbtx.commit().await?;
            } else {
                tracing::info!(?name, "already initialized");
            }
            while let Some(events) = rx.recv().await {
                let mut dbtx = state_cp.begin_transaction().await?;
                let last_height = events.last_height;
                if index_height >= Height::from(last_height) {
                    tracing::info!(
                        first = events.first_height,
                        last = events.last_height,
                        index_name = &name,
                        "skipping batch"
                    );
                    continue;
                }
                tracing::info!(
                    first = events.first_height,
                    last = events.last_height,
                    index_name = &name,
                    "indexing batch"
                );
                index.index_batch(&mut dbtx, events).await?;
                tracing::debug!(index_name = &name, "committing batch");
                IndexingState::update_index_height(&mut dbtx, &name, Height::from(last_height))
                    .await?;

                dbtx.commit().await?;
            }
            Ok(())
        });
    }

    let state_cp = state.clone();
    tasks.spawn(async move {
        let mut height = lowest_index_height.next();
        while height < src_height {
            let first = height;
            let (last, next_height) = first.advance(DEFAULT_BATCH_SIZE, src_height);
            height = next_height;
            tracing::debug!(?first, ?last, "fetching batch");
            let events = state_cp.event_batch(first, last).await?;
            tracing::info!(?first, ?last, "sending batch");
            for tx in &txs {
                tx.send(events.clone()).await?;
            }
        }
        Ok(())
    });

    while let Some(res) = tasks.join_next().await {
        res??;
    }
    Ok(())
}

pub struct Indexer {
    opts: Options,
    indices: Vec<Arc<dyn AppView>>,
}

impl Indexer {
    pub fn new(opts: Options) -> Self {
        Self {
            opts,
            indices: Vec::new(),
        }
    }

    pub fn with_index(mut self, index: Box<dyn AppView + 'static>) -> Self {
        self.indices.push(Arc::from(index));
        self
    }

    pub fn with_default_tracing(self) -> Self {
        tracing_subscriber::fmt::init();
        self
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        tracing::info!(?self.opts);
        let Self {
            opts:
                Options {
                    src_database_url,
                    dst_database_url,
                    chain_id: _,
                    poll_ms,
                    genesis_json,
                },
            indices: indexes,
        } = self;

        let state = IndexingState::init(&src_database_url, &dst_database_url).await?;
        let genesis: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(genesis_json)
                .context("error reading provided genesis.json file")?,
        )
        .context("error parsing provided genesis.json file")?;
        let app_state = Arc::new(
            genesis
                .get("app_state")
                .ok_or_else(|| anyhow::anyhow!("genesis missing app_state"))?
                .clone(),
        );
        loop {
            catchup(&state, indexes.as_slice(), app_state.clone()).await?;
            tokio::time::sleep(poll_ms).await;
        }
    }
}
