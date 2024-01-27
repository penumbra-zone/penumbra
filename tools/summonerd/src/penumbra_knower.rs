use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use camino::Utf8Path;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use penumbra_keys::{Address, FullViewingKey};
use penumbra_num::Amount;
use penumbra_view::{Storage, ViewService};
use tokio::sync::Mutex;
use url::Url;

/// The amount of time to wait for a new block before restarting the view service.
const RESTART_TIME_SECS: u64 = 20;

/// The actions the watcher can ask us to do.
#[derive(Debug, Clone, Copy)]
enum WatcherAdvice {
    DoNothing,
    RestartTheViewService,
}

/// A stateful adviser on restarting view services.
#[derive(Debug, Clone, Copy)]
struct Watcher {
    sync_height: u64,
    sync_time: Option<Instant>,
}

impl Watcher {
    pub fn new() -> Self {
        Self {
            sync_height: 0,
            sync_time: None,
        }
    }

    pub fn what_should_i_do(&mut self, sync_height: Option<u64>, now: Instant) -> WatcherAdvice {
        let sync_height = sync_height.unwrap_or(0u64);
        if sync_height > self.sync_height {
            self.sync_height = sync_height;
            self.sync_time = Some(now);
            return WatcherAdvice::DoNothing;
        }
        match self.sync_time {
            Some(then) if now.duration_since(then) >= Duration::from_secs(RESTART_TIME_SECS) => {
                WatcherAdvice::RestartTheViewService
            }
            _ => WatcherAdvice::DoNothing,
        }
    }
}

/// Knows things about a running penumbra system, requires internet connectivity
#[derive(Clone)]
pub struct PenumbraKnower {
    // Nota bene that this is the storage from the view service, and is how
    // we get the specific information we need, as this will get populated
    // by the view service.
    storage: Storage,
    // The node the view service will use.
    node: Url,
    // Not sure if storing this is necessary, but seems like a good idea to avoid things getting
    // dropped on the floor
    view: Arc<Mutex<ViewService>>,
    // Some state for calculating whether ro restart the view service.
    watcher: Arc<Mutex<Watcher>>,
}

impl PenumbraKnower {
    /// Create the knower, loading or initializing the storage it uses for penumbra data.
    ///
    /// This name has been passed down through generations of structs.
    pub async fn load_or_initialize(
        storage_path: impl AsRef<Utf8Path>,
        fvk: &FullViewingKey,
        node: Url,
    ) -> Result<Self> {
        let storage = Storage::load_or_initialize(Some(storage_path), fvk, node.clone()).await?;
        let view = ViewService::new(storage.clone(), node.clone()).await?;
        Ok(Self {
            storage,
            node,
            view: Arc::new(Mutex::new(view)),
            watcher: Arc::new(Mutex::new(Watcher::new())),
        })
    }

    async fn restart_view_service_if_necesary(&self) -> Result<()> {
        let sync_height = self.storage.last_sync_height().await?;
        match self
            .watcher
            .lock()
            .await
            .what_should_i_do(sync_height, Instant::now())
        {
            WatcherAdvice::DoNothing => {}
            WatcherAdvice::RestartTheViewService => {
                tracing::info!("restarting the view service");
                let new_view = ViewService::new(self.storage.clone(), self.node.clone()).await?;
                let mut view = self.view.lock().await;
                std::mem::replace(&mut *view, new_view).abort().await;
            }
        }
        Ok(())
    }

    pub async fn total_amount_sent_to_me(&self, by: &Address) -> Result<Amount> {
        self.restart_view_service_if_necesary().await?;

        let notes = self.storage.notes_by_sender(by).await?;
        let what_i_want = STAKING_TOKEN_ASSET_ID.to_owned();
        let mut total = Amount::zero();
        for note in &notes {
            if note.note.asset_id() != what_i_want {
                continue;
            }
            total = total.saturating_add(&note.note.amount());
        }
        Ok(total)
    }
}
