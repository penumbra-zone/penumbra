use anyhow::Result;
use camino::Utf8Path;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;
use penumbra_keys::{Address, FullViewingKey};
use penumbra_num::Amount;
use penumbra_view::{Storage, ViewService};
use url::Url;

/// Knows things about a running penumbra system, requires internet connectivity
#[derive(Clone)]
pub struct PenumbraKnower {
    // Nota bene that this is the storage from the view service, and is how
    // we get the specific information we need, as this will get populated
    // by the view service.
    storage: Storage,
    // Not sure if storing this is necessary, but seems like a good idea to avoid things getting
    // dropped on the floor
    _view: ViewService,
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
        let view = ViewService::new(storage.clone(), node).await?;
        Ok(Self {
            storage,
            _view: view,
        })
    }

    pub async fn total_amount_sent_to_me(&self, by: &Address) -> Result<Amount> {
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
