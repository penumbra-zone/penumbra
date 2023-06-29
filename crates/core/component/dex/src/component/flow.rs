use std::ops::{Deref, DerefMut};

use penumbra_crypto::Amount;

// Tuple represents:
// ((amount of asset 1 being exchanged for asset 2),
//  (amount of asset 2 being exchanged for asset 1))
#[derive(Default, Clone)]
pub struct SwapFlow((Amount, Amount));

impl Deref for SwapFlow {
    type Target = (Amount, Amount);

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SwapFlow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
