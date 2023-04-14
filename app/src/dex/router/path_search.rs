use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::{asset, fixpoint::U128x128};
use penumbra_storage::{StateDelta, StateRead};

#[async_trait]
pub trait PathSearch: StateRead + Clone + 'static {
    /// Find the best route from `src` to `dst`, also returning the spill price for the next-best route, if one exists.
    async fn best_route_with_spill_price(
        &self,
        src: asset::Id,
        dst: asset::Id,
    ) -> Result<(Vec<asset::Id>, Option<U128x128>)> {
        // Work in a new stack of state changes, which we can completely discard
        // at the end of routing
        let mut state = StateDelta::new(self.clone());
        // do routing
        // ...
        // discard all state changes made during routing
        // note: we don't return a `Path`, which will be an internal impl detail.
        // (best_path, spill_price)
        todo!()
    }
}
