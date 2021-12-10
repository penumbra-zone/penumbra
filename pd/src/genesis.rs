use ark_ff::Zero;
use decaf377::Fq;
use serde::{Deserialize, Serialize};

use penumbra_crypto::{asset::Denom, Address, Note, Value};

use crate::staking::Validator;

/// A (transparent) genesis allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub amount: u64,
    pub denom: String,
    pub address: Address,
}

impl Allocation {
    /// Obtain a note corresponding to this allocation.
    ///
    /// Note: to ensure determinism, this uses a zero blinding factor when
    /// creating the note. This is fine, because the genesis allocations are
    /// already public.
    pub fn note(&self) -> Result<Note, anyhow::Error> {
        Note::from_parts(
            *self.address.diversifier(),
            *self.address.transmission_key(),
            Value {
                amount: self.amount,
                asset_id: Denom(self.denom.clone()).id(),
            },
            Fq::zero(),
        )
        .map_err(Into::into)
    }
}

/// The application state at genesis.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppState {
    /// The initial token allocations.
    pub allocations: Vec<Allocation>,
    /// The number of blocks in each epoch.
    pub epoch_duration: u64,
    /// The initial validator set.
    pub validators: Vec<Validator>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            epoch_duration: 8640,
            allocations: Vec::default(),
            validators: Vec::default(),
        }
    }
}
