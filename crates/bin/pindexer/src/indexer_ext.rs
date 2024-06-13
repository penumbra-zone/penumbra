pub trait IndexerExt: Sized {
    fn with_default_penumbra_app_views(self) -> Self;
}

impl IndexerExt for cometindex::Indexer {
    fn with_default_penumbra_app_views(self) -> Self {
        self.with_index(crate::shielded_pool::fmd::ClueSet {})
    }
}
