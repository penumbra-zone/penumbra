use ark_ff::Zero;
use decaf377::Fq;
use penumbra_crypto::{asset, Address, Note, Value};
use penumbra_stake::Validator;
use serde::{Deserialize, Serialize};

/// A (transparent) genesis allocation.
#[derive(Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub amount: u64,
    pub denom: String,
    pub address: Address,
}

// Implement Debug manually so we can use the Display impl for the address,
// rather than dumping all the internal address components.
impl std::fmt::Debug for Allocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allocation")
            .field("amount", &self.amount)
            .field("denom", &self.denom)
            .field("address", &self.address.to_string())
            .finish()
    }
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
                asset_id: asset::REGISTRY
                    .parse_denom(&self.denom)
                    .ok_or_else(|| anyhow::anyhow!("invalid denomination"))?
                    .id(),
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
