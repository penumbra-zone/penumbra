use std::sync::Arc;

use penumbra_sdk_asset::asset;
use penumbra_sdk_num::fixpoint::U128x128;

use crate::DexParameters;

#[derive(Debug, Clone)]
pub struct RoutingParams {
    pub price_limit: Option<U128x128>,
    pub fixed_candidates: Arc<Vec<asset::Id>>,
    pub max_hops: usize,
}

impl RoutingParams {
    pub fn with_extra_candidates(self, iter: impl IntoIterator<Item = asset::Id>) -> Self {
        let mut fixed_candidates: Vec<_> = (*self.fixed_candidates).clone();
        fixed_candidates.extend(iter);

        Self {
            fixed_candidates: Arc::new(fixed_candidates),
            ..self
        }
    }

    /// Clamps the spill price to the price limit and returns whether or not it was clamped.
    pub fn clamp_to_limit(&self, spill_price: Option<U128x128>) -> (Option<U128x128>, bool) {
        match (spill_price, self.price_limit) {
            (Some(spill_price), Some(price_limit)) => {
                if spill_price > price_limit {
                    (Some(price_limit), true)
                } else {
                    (Some(spill_price), false)
                }
            }
            (Some(spill_price), None) => (Some(spill_price), false),
            (None, Some(price_limit)) => (Some(price_limit), true),
            (None, None) => (None, false),
        }
    }
}

impl From<DexParameters> for RoutingParams {
    fn from(
        DexParameters {
            fixed_candidates,
            max_hops,
            ..
        }: DexParameters,
    ) -> Self {
        Self {
            fixed_candidates: Arc::new(fixed_candidates),
            max_hops: max_hops as usize,
            price_limit: None,
        }
    }
}
