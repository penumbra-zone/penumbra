use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use anyhow::Result;
use penumbra_keys::Address;
use penumbra_proto::tools::summoning::v1alpha1::CeremonyCrs;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Storage {
    // TODO: This should have methods for persisting all of the state of the coordinator,
    // using a sqlite database.
    crs: Arc<Mutex<CeremonyCrs>>,
    slot: Arc<AtomicU64>
}

impl Storage {
    pub fn new() -> Self {
        Self {
            crs: Arc::new(Mutex::new(CeremonyCrs::default())),
            slot: Arc::new(AtomicU64::new(0))
        }
    }

    pub async fn can_contribute(&self, address: Address) -> Result<()> {
        // Criteria:
        // - Not banned
        // - Bid more than min amount
        // - Hasn't already contributed
        Ok(())
    }

    pub async fn current_crs(&self) -> Result<CeremonyCrs> {
        Ok(self.crs.lock().await.clone())
    }

    // TODO: Add other stuff here
    pub async fn commit_contribution(&self, contributor: Address, crs: &CeremonyCrs) -> Result<()> {
        // TODO: Do.
        *self.crs.lock().await = crs.clone();
        self.slot.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub async fn current_slot(&self) -> Result<u64> {
        Ok(self.slot.load(Ordering::SeqCst))
    }
}
