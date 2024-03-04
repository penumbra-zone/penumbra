use ark_ff::{One, Zero};
use decaf377::{Element, Fr};
pub use frost_core::{Ciphersuite, Field, FieldError, Group, GroupError};
use rand_core;

use crate::hash::Hasher;

#[derive(Copy, Clone)]
pub struct Decaf377ScalarField;

impl Field for Decaf377ScalarField {
    type Scalar = Fr;

    type Serialization = Vec<u8>;

    fn zero() -> Self::Scalar {
        Fr::zero()
    }

    fn one() -> Self::Scalar {
        Fr::one()
    }

    fn invert(scalar: &Self::Scalar) -> Result<Self::Scalar, FieldError> {
        scalar.inverse().ok_or(FieldError::InvalidZeroScalar)
    }

    fn random<R: rand_core::RngCore + rand_core::CryptoRng>(rng: &mut R) -> Self::Scalar {
        Fr::rand(rng)
    }

    fn serialize(scalar: &Self::Scalar) -> Self::Serialization {
        scalar.to_bytes().to_vec()
    }

    fn little_endian_serialize(scalar: &Self::Scalar) -> Self::Serialization {
        Self::serialize(scalar)
    }

    fn deserialize(buf: &Self::Serialization) -> Result<Self::Scalar, FieldError> {
        Fr::from_bytes_checked(
            &TryInto::<[u8; 32]>::try_into(buf.clone()).map_err(|_| FieldError::MalformedScalar)?,
        )
        .map_err(|_| FieldError::MalformedScalar)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Decaf377Group;

impl Group for Decaf377Group {
    type Field = Decaf377ScalarField;

    type Element = Element;

    type Serialization = Vec<u8>;

    fn cofactor() -> <Self::Field as Field>::Scalar {
        Fr::one()
    }

    fn identity() -> Self::Element {
        Element::default()
    }

    fn generator() -> Self::Element {
        decaf377::Element::GENERATOR
    }

    fn serialize(element: &Self::Element) -> Self::Serialization {
        element.vartime_compress().0.to_vec()
    }

    fn deserialize(buf: &Self::Serialization) -> Result<Self::Element, GroupError> {
        decaf377::Encoding(
            TryInto::<[u8; 32]>::try_into(buf.clone()).map_err(|_| GroupError::MalformedElement)?,
        )
        .vartime_decompress()
        .map_err(|_| GroupError::MalformedElement)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Decaf377Rdsa;

#[allow(non_snake_case)]
impl Ciphersuite for Decaf377Rdsa {
    const ID: &'static str = "decaf377-rdsa";

    type Group = Decaf377Group;

    type HashOutput = [u8; 32];

    type SignatureSerialization = Vec<u8>;

    fn H1(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
        Hasher::default().update(b"rho").update(m).finalize_scalar()
    }

    fn H2(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
        Hasher::challenge().update(m).finalize_scalar()
    }

    fn H3(m: &[u8]) -> <<Self::Group as Group>::Field as Field>::Scalar {
        Hasher::default()
            .update(b"nonce")
            .update(m)
            .finalize_scalar()
    }

    fn H4(m: &[u8]) -> Self::HashOutput {
        Hasher::default().update(b"msg").update(m).finalize()
    }

    fn H5(m: &[u8]) -> Self::HashOutput {
        Hasher::default().update(b"com").update(m).finalize()
    }

    fn HDKG(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
        Some(Hasher::default().update(b"dkg").update(m).finalize_scalar())
    }

    fn HID(m: &[u8]) -> Option<<<Self::Group as Group>::Field as Field>::Scalar> {
        Some(Hasher::default().update(b"id").update(m).finalize_scalar())
    }
}
