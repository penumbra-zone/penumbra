//! Validator-related facilities.
//
//  TODO(kate):
//    - `Into`, implementation for converting into a tendermint::..Validator
//    - method for getting a key as a tendermint::..SigningKey
//
//  TODO(kate): implement the proposer selection
//    - https://github.com/cometbft/cometbft/blob/main/spec/consensus/proposer-selection.md#basic-algorithm

use {rand_core::OsRng, tendermint::vote};

/// A validator set.
#[allow(dead_code)] // XXX(kate)
pub struct Validators {
    pub(crate) current: Vec<Validator>,
    // TODO(kate): we may want these fields too.
    //   - next: Vec<Validator>,
    //   - last: Vec<Validator>,
    //   - last_height_validators_changed: block::Height,
}

/// A single validator.
///
/// This is a [`tendermint::abci::types::Validator`], but with signing keys held in-memory.
///
/// See the [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#validator)
/// for more information on validators.
#[allow(dead_code)] // XXX(kate)
pub struct Validator {
    pub(crate) name: Option<String>,
    /// The validator's address (the first 20 bytes of `SHA256(public_key)`).
    //
    //  TODO(kate): this should be an `account::Id` it seems? see `validator::Info`.
    pub(crate) address: [u8; 20],
    /// The voting power of the validator.
    pub(crate) power: vote::Power,
    /// The validator's (private) signing key.
    signing_key: ed25519_consensus::SigningKey,
    /// The validator's (public) verification key.
    pub(crate) verification_key: ed25519_consensus::VerificationKey,
}

// === impl Validators ===

impl Validators {
    /// Returns a validator set consisting of a single [`Validator`].
    pub fn single() -> Self {
        let power = 100_u32.into();
        Self {
            current: vec![Validator::new(power)],
        }
    }

    /// Returns a new [`Validators`] using the provided collection of validators.
    pub fn new(validators: Vec<Validator>) -> Self {
        Self {
            current: validators,
        }
    }
}

/// A single in-memory [`Validator`] is used by default.
impl Default for Validators {
    fn default() -> Self {
        Self::single()
    }
}

/// A validator set may be [`collect()`][Iterator::collect]ed from an iterator.
impl FromIterator<Validator> for Validators {
    fn from_iter<T: IntoIterator<Item = Validator>>(iter: T) -> Self {
        let current = iter.into_iter().collect();
        Self { current }
    }
}

// === impl Validator ===

impl Validator {
    /// Returns a [`Validator`] with the designated voting power.
    ///
    /// NB: This will generate a [`ed25519_consensus::SigningKey`] for this validator.
    pub fn new(power: vote::Power) -> Self {
        let signing_key = ed25519_consensus::SigningKey::new(OsRng);
        let verification_key = signing_key.verification_key();
        let address = Self::address_from_public_key(&verification_key);
        Validator {
            name: None,
            address,
            power,
            signing_key,
            verification_key,
        }
    }

    /// Returns this validator, with the given name.
    pub fn with_name(self, name: impl AsRef<str>) -> Self {
        let name = name.as_ref().to_owned();
        Self {
            name: Some(name),
            ..self
        }
    }

    // TODO(kate): tendermint-rs says addresses are the first 20 bytes of the sha256 hash of the
    // public key. track down where this is described in the spec.
    fn address_from_public_key(public_key: impl AsRef<[u8]>) -> [u8; 20] {
        use sha2::{Digest as _, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        hash[..20].try_into().expect("")
    }

    /// Returns [`Info`][tendermint::validator::Info] about this validator.
    pub fn info(&self) -> tendermint::validator::Info {
        let Self {
            name,
            address,
            power,
            verification_key,
            ..
        } = self;
        let address = address.to_vec().try_into().unwrap(/*XXX(kate): add error handling*/);
        let pub_key: tendermint::PublicKey = {
            let bytes = verification_key.to_bytes();
            tendermint::crypto::ed25519::VerificationKey::try_from(bytes.as_slice())
                .unwrap()
                // XXX(kate): file a PR upstream here. there is already a constructor to
                // create these from a fixed-size array, we shouldn't need to perform another
                // length check if we have the ed25519_consensus key in hand.
                // https://rustdoc.penumbra.zone/main/src/tendermint/crypto/ed25519/verification_key.rs.html#4
                .into()
        };
        tendermint::validator::Info {
            address,
            pub_key,
            power: *power,
            name: name.clone(),
            proposer_priority: 1.into(),
        }
    }
}

// === unit tests ===

#[cfg(test)]
mod validator_tests {
    use super::*;
    #[test]
    fn single_validator_can_be_created() {
        let _ = Validators::single();
    }
}
