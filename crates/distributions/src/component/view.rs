use anyhow::Result;
use async_trait::async_trait;

use penumbra_crypto::{asset, Amount, Value};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use super::state_key;

#[async_trait]
pub trait StateReadExt: StateRead {}

impl<T> StateReadExt for T where T: StateRead + ?Sized {}

#[async_trait]
pub trait StateWriteExt: StateWrite {}

impl<T> StateWriteExt for T where T: StateWrite + ?Sized {}
