use std::str::FromStr;

pub trait IndexerExt: Sized {
    fn with_default_penumbra_app_views(self) -> Self;
}

impl IndexerExt for cometindex::Indexer {
    fn with_default_penumbra_app_views(self) -> Self {
        self.with_index(Box::new(crate::block::Block {}))
            .with_index(Box::new(crate::stake::ValidatorSet {}))
            .with_index(Box::new(crate::stake::Slashings {}))
            .with_index(Box::new(crate::stake::DelegationTxs {}))
            .with_index(Box::new(crate::stake::UndelegationTxs {}))
            .with_index(Box::new(crate::governance::GovernanceProposals {}))
            .with_index(Box::new(crate::dex_ex::Component::new(
                penumbra_sdk_asset::asset::Id::from_str(
                    // USDC
                    "passet1w6e7fvgxsy6ccy3m8q0eqcuyw6mh3yzqu3uq9h58nu8m8mku359spvulf6",
                )
                .expect("should be able to parse passet"),
                1000.0 * 1_000_000.0,
            )))
            .with_index(Box::new(crate::supply::Component::new()))
            .with_index(Box::new(crate::ibc::Component::new()))
            .with_index(Box::new(crate::insights::Component::new(
                penumbra_sdk_asset::asset::Id::from_str(
                    // USDC
                    "passet1w6e7fvgxsy6ccy3m8q0eqcuyw6mh3yzqu3uq9h58nu8m8mku359spvulf6",
                )
                .ok(),
            )))
    }
}
