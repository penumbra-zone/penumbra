mod indexing_state;

use crate::{
    index::{EventBatch, EventBatchContext},
    opt::Options,
    AppView,
};
use anyhow::{Context, Result};
use indexing_state::{Height, IndexState, IndexingManager};
use std::sync::Arc;
use tokio::{sync::mpsc, task::JoinSet};

async fn reset_index_if_necessary(
    index: &dyn AppView,
    manager: &IndexingManager,
    dbtx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<()> {
    let name = index.name();
    let state = manager.index_state(&name).await?;
    let version = index.version();
    if version < state.version {
        // My thinking is that the only reason this can happen is that:
        // a) Someone accidentally decreased the version in their AppView.
        // b) For some reason, we're running the wrong version of the consuming pindexer against a DB.
        anyhow::bail!(
            r#"
Current version for index {name} {version:?} is lower than that recorded in the state: {0:?}.
Are you running the right version of the code?
If so, maybe there's a bug in this particular index.
        "#,
            state.version
        );
    } else if version > state.version {
        tracing::info!(?name, old_version = ?state.version, new_version = ?version, "resetting index");
        index.reset(dbtx).await?;
        IndexingManager::update_index_state(
            dbtx,
            &name,
            IndexState {
                height: Height::default(),
                version,
            },
        )
        .await?;
    }
    Ok(())
}

/// Attempt to catch up to the latest indexed block.
///
/// Returns whether or not we've caught up.
#[tracing::instrument(skip_all)]
async fn catchup(
    manager: &IndexingManager,
    indices: &[Arc<dyn AppView>],
    genesis: Arc<serde_json::Value>,
) -> anyhow::Result<bool> {
    if indices.len() <= 0 {
        tracing::info!(why = "no indices", "catchup completed");
        return Ok(true);
    }

    let (src_height, index_states) =
        tokio::try_join!(manager.src_height(), manager.index_states())?;
    tracing::info!(?src_height, ?index_states, "catchup status");
    let lowest_index_height = index_states
        .values()
        .map(|x| x.height)
        .min()
        .unwrap_or_default();
    if lowest_index_height >= src_height {
        tracing::info!(why = "already caught up", "catchup completed");
        return Ok(true);
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
        let index_state = index_states.get(&name).copied().unwrap_or_default();
        let manager_cp = manager.clone();
        let genesis_cp = genesis.clone();
        tasks.spawn(async move {
            if index_state.height == Height::default() {
                tracing::info!(?name, "initializing index");
                let mut dbtx = manager_cp.begin_transaction().await?;
                index.init_chain(&mut dbtx, &genesis_cp).await?;
                tracing::info!(?name, "finished initialization");
                let new_state = IndexState {
                    version: index_state.version,
                    height: Height::default(),
                };
                IndexingManager::update_index_state(&mut dbtx, &name, new_state).await?;
                dbtx.commit().await?;
            } else {
                tracing::info!(?name, "already initialized");
            }
            while let Some(mut events) = rx.recv().await {
                // We only ever want to index events past our current height.
                // We might receive a batch with more events because other indices are behind us.
                events.start_later(index_state.height.next().into());
                if events.empty() {
                    tracing::info!(
                        first = events.first_height(),
                        last = events.last_height(),
                        index_name = &name,
                        "skipping batch"
                    );
                    continue;
                }
                tracing::info!(
                    first = events.first_height(),
                    last = events.last_height(),
                    index_name = &name,
                    "indexing batch"
                );
                let last_height = events.last_height();
                let mut dbtx = manager_cp.begin_transaction().await?;
                let context = EventBatchContext {
                    is_last: last_height >= u64::from(src_height),
                };
                index.index_batch(&mut dbtx, events, context).await?;
                tracing::debug!(index_name = &name, "committing batch");
                let new_state = IndexState {
                    version: index.version(),
                    height: Height::from(last_height),
                };
                IndexingManager::update_index_state(&mut dbtx, &name, new_state).await?;

                dbtx.commit().await?;
            }
            Ok(())
        });
    }

    let manager_cp = manager.clone();
    tasks.spawn(async move {
        let mut height = lowest_index_height.next();
        while height <= src_height {
            let first = height;
            let (last, next_height) = first.advance(DEFAULT_BATCH_SIZE, src_height);
            height = next_height;
            tracing::debug!(?first, ?last, "fetching batch");
            let events = manager_cp.event_batch(first, last).await?;
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
    Ok(false)
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
                    exit_on_catchup,
                    integrity_checks_only,
                },
            indices,
        } = self;
        crate::integrity::integrity_check(&src_database_url)
            .await
            .context("while running integrity checks")?;
        if integrity_checks_only {
            return Ok(());
        }

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

        let manager = IndexingManager::init(&src_database_url, &dst_database_url).await?;
        {
            let mut dbtx = manager.begin_transaction().await?;
            for index in &indices {
                reset_index_if_necessary(index.as_ref(), &manager, &mut dbtx).await?;
                index.on_startup(&mut dbtx).await?;
            }
            dbtx.commit().await?;
        }

        loop {
            let caught_up = catchup(&manager, indices.as_slice(), app_state.clone()).await?;
            if exit_on_catchup && caught_up {
                tracing::info!("catchup completed, exiting as requested");
                return Ok(());
            }
            tokio::time::sleep(poll_ms).await;
        }
    }
}
