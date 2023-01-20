use async_trait::async_trait;
use penumbra_storage::StateWrite;

/// Manages liquidity positions within the chain state.
#[async_trait]
pub trait PositionManager: StateWrite {}

impl<T: StateWrite + ?Sized> PositionManager for T {}
