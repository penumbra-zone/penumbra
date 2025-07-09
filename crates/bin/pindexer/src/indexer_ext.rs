pub trait IndexerExt: Sized {
    fn with_default_penumbra_app_views(self, options: &crate::Options) -> Self;
}

impl IndexerExt for cometindex::Indexer {
    fn with_default_penumbra_app_views(self, options: &crate::Options) -> Self {
        self.with_index(Box::new(crate::block::Block {}))
            .with_index(Box::new(crate::stake::ValidatorSet {}))
            .with_index(Box::new(crate::stake::Slashings {}))
            .with_index(Box::new(crate::stake::DelegationTxs {}))
            .with_index(Box::new(crate::stake::UndelegationTxs {}))
            .with_index(Box::new(crate::governance::GovernanceProposals {}))
            .with_index(Box::new(crate::dex_ex::Component::new(
                options.indexing_denom,
                options.dex_ex_min_liquidity as f64,
                options.dex_ex_ignore_arb,
            )))
            .with_index(Box::new(crate::supply::Component::new()))
            .with_index(Box::new(crate::ibc::Component::new()))
            .with_index(Box::new(crate::insights::Component::new(Some(
                options.indexing_denom,
            ))))
            .with_index(Box::new(crate::lqt::Lqt::new(options.block_time_s)))
    }
}
