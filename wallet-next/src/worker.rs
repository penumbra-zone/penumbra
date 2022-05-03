use crate::Storage;

pub struct Worker {
    storage: Storage,
    // TODO: notifications (see TODOs on WalletService)
}

impl Worker {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn sync_to_latest(&self) -> Result<u64, anyhow::Error> {
        // Do a single sync run, up to whatever the latest block height is
        todo!()
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        loop {
            self.sync_to_latest().await?;

            // TODO 1: randomize sleep interval within some range?
            // TODO 2: use websockets to be notified on new block
            tokio::time::sleep(std::time::Duration::from_millis(1729)).await;
        }
    }
}
