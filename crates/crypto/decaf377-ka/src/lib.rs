#![deny(clippy::unwrap_used)]
// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::convert::{TryFrom, TryInto};

use decaf377::{self};
use rand_core::{CryptoRng, RngCore};
use zeroize::Zeroize;

/// A public key sent to the counterparty in the key agreement protocol.
///
/// This is a refinement type around `[u8; 32]` that marks the bytes as being a
/// public key.  Not all 32-byte arrays are valid public keys; invalid public
/// keys will error during key agreement.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Public(pub [u8; 32]);

/// A secret key used to perform key agreement using the counterparty's public key.
#[derive(Clone, Zeroize, PartialEq, Eq)]
#[zeroize(drop)]
pub struct Secret(decaf377::Fr);

/// The shared secret derived at the end of the key agreement protocol.
#[derive(PartialEq, Eq, Clone, Zeroize)]
#[zeroize(drop)]
pub struct SharedSecret(pub [u8; 32]);

/// An error during key agreement.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid public key")]
    InvalidPublic(Public),
    #[error("Invalid secret key")]
    InvalidSecret,
    #[error("Supplied bytes are incorrect length")]
    SliceLenError,
}

impl Secret {
    /// Generate a new secret key using `rng`.
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        Self(decaf377::Fr::rand(rng))
    }

    /// Use the supplied field element as the secret key directly.
    ///
    /// # Warning
    ///
    /// This function exists to allow custom key derivation; it's the caller's
    /// responsibility to ensure that the input was generated securely.
    pub fn new_from_field(sk: decaf377::Fr) -> Self {
        Self(sk)
    }

    /// Derive a public key for this secret key, using the conventional
    /// `decaf377` generator.
    pub fn public(&self) -> Public {
        self.diversified_public(&decaf377::Element::GENERATOR)
    }

    /// Derive a diversified public key for this secret key, using the provided
    /// `diversified_generator`.
    ///
    /// Since key agreement does not depend on the basepoint, only on the secret
    /// key and the public key, a single secret key can correspond to many
    /// different (unlinkable) public keys.
    pub fn diversified_public(&self, diversified_generator: &decaf377::Element) -> Public {
        Public((self.0 * diversified_generator).vartime_compress().into())
    }

    /// Perform key agreement with the provided public key.
    ///
    /// Fails if the provided public key is invalid.
    pub fn key_agreement_with(&self, other: &Public) -> Result<SharedSecret, Error> {
        let pk = decaf377::Encoding(other.0)
            .vartime_decompress()
            .map_err(|_| Error::InvalidPublic(*other))?;

        Ok(SharedSecret((self.0 * pk).vartime_compress().into()))
    }

    /// Convert this shared secret to bytes.
    ///
    /// Convenience wrapper around an [`Into`] impl.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.into()
    }
}

impl std::fmt::Debug for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "decaf377_ka::Public({})",
            hex::encode(&self.0[..])
        ))
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_bytes();
        f.write_fmt(format_args!(
            "decaf377_ka::Secret({})",
            hex::encode(&bytes[..])
        ))
    }
}

impl std::fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "decaf377_ka::SharedSecret({})",
            hex::encode(&self.0[..])
        ))
    }
}

impl TryFrom<&[u8]> for Public {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Public, Error> {
        let bytes: [u8; 32] = slice.try_into().map_err(|_| Error::SliceLenError)?;
        Ok(Public(bytes))
    }
}

impl TryFrom<&[u8]> for Secret {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Secret, Error> {
        let bytes: [u8; 32] = slice.try_into().map_err(|_| Error::SliceLenError)?;
        bytes.try_into()
    }
}

impl TryFrom<[u8; 32]> for Secret {
    type Error = Error;
    fn try_from(bytes: [u8; 32]) -> Result<Secret, Error> {
        let x = decaf377::Fr::from_bytes_checked(&bytes).map_err(|_| Error::InvalidSecret)?;
        Ok(Secret(x))
    }
}

impl TryFrom<[u8; 32]> for SharedSecret {
    type Error = Error;
    fn try_from(bytes: [u8; 32]) -> Result<SharedSecret, Error> {
        decaf377::Encoding(bytes)
            .vartime_decompress()
            .map_err(|_| Error::InvalidSecret)?;

        Ok(SharedSecret(bytes))
    }
}

impl From<&Secret> for [u8; 32] {
    fn from(s: &Secret) -> Self {
        s.0.to_bytes()
    }
}
