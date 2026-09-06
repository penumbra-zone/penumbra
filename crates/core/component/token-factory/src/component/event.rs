//! Event emission for token factory actions.

use penumbra_sdk_proto::core::component::token_factory::v1 as pb;

use crate::{ActionTokenFactoryCreate, ActionTokenFactoryMint};

/// Create an event for token creation.
pub fn token_factory_create(action: &ActionTokenFactoryCreate) -> pb::EventTokenFactoryCreate {
    pb::EventTokenFactoryCreate {
        id: Some(action.nonce.into()),
        metadata: Some(action.metadata.clone().into()),
        initial_supply: Some(action.initial_supply.into()),
        enable_mint: action.enable_mint,
    }
}

/// Create an event for token minting.
pub fn token_factory_mint(action: &ActionTokenFactoryMint) -> pb::EventTokenFactoryMint {
    pb::EventTokenFactoryMint {
        id: Some(action.token_id().clone().into()),
        seq: action.current_seq(),
        amount: Some(action.amount().into()),
    }
}
