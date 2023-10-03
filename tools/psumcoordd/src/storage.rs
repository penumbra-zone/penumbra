use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anyhow::Result;
use penumbra_keys::Address;
use penumbra_proof_setup::all::{Phase2CeremonyCRS, Phase2CeremonyContribution};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Storage {
    // TODO: This should have methods for persisting all of the state of the coordinator,
    // using a sqlite database.
    crs: Arc<Mutex<Phase2CeremonyCRS>>,
    slot: Arc<AtomicU64>,
    root: Phase2CeremonyCRS,
}

impl Storage {
    pub fn new(root: Phase2CeremonyCRS) -> Self {
        Self {
            crs: Arc::new(Mutex::new(root.clone())),
            slot: Arc::new(AtomicU64::new(0)),
            root,
        }
    }

    pub async fn root(&self) -> Result<Phase2CeremonyCRS> {
        Ok(self.root.clone())
    }

    pub async fn can_contribute(&self, _address: Address) -> Result<()> {
        // Criteria:
        // - Not banned
        // - Bid more than min amount
        // - Hasn't already contributed
        Ok(())
    }

    pub async fn current_crs(&self) -> Result<Phase2CeremonyCRS> {
        Ok(self.crs.lock().await.clone())
    }

    // TODO: Add other stuff here
    pub async fn commit_contribution(
        &self,
        _contributor: Address,
        contribution: &Phase2CeremonyContribution,
    ) -> Result<()> {
        *self.crs.lock().await = contribution.new_elements();
        self.slot.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub async fn current_slot(&self) -> Result<u64> {
        Ok(self.slot.load(Ordering::SeqCst))
    }
}
