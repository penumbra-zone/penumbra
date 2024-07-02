use blake2b_simd::{Params, State};
use decaf377::Fr;

pub struct Hasher {
    state: State,
}

impl Default for Hasher {
    fn default() -> Self {
        let state = Params::new()
            .hash_length(64)
            .personal(b"FROST-decaf377")
            .to_state();
        Self { state }
    }
}

impl Hasher {
    /// Create a hasher which matches the challenge generation of decaf377-rdsa
    pub fn challenge() -> Self {
        let state = Params::new()
            .hash_length(64)
            .personal(b"decaf377-rdsa---")
            .to_state();
        Self { state }
    }
}

impl Hasher {
    /// Add `data` to the hash, and return `Self` for chaining.
    pub fn update(&mut self, data: impl AsRef<[u8]>) -> &mut Self {
        self.state.update(data.as_ref());
        self
    }

    /// Consume `self` to compute the hash output, and convert it to a scalar.
    pub fn finalize_scalar(&self) -> Fr {
        Fr::from_le_bytes_mod_order(self.state.finalize().as_array())
    }

    /// Consume `self` to compute the hash output.
    pub fn finalize(&self) -> [u8; 32] {
        self.state.finalize().as_bytes()[..32]
            .try_into()
            .expect("failed to convert blake2b hash to array")
    }
}
