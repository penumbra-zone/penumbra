use std::ops::Deref;

use ark_ff::PrimeField;
use decaf377::Fq;
use decaf377::Fr;
use once_cell::sync::Lazy;
use penumbra_proto::crypto as pb;
use penumbra_proto::Protobuf;

use crate::Value;

impl Value {
    #[allow(non_snake_case)]
    pub fn commit(&self, blinding: Fr) -> Commitment {
        let G_v = self.asset_id.value_generator();
        let H = VALUE_BLINDING_GENERATOR.deref();

        let v = Fr::from(self.amount);
        let C = v * G_v + blinding * H;

        Commitment(C)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Commitment(pub decaf377::Element);

impl Commitment {
    pub fn to_bytes(&self) -> [u8; 32] {
        (*self).into()
    }
}

pub static VALUE_BLINDING_GENERATOR: Lazy<decaf377::Element> = Lazy::new(|| {
    let s = Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"decaf377-rdsa-binding").as_bytes());
    decaf377::Element::encode_to_curve(&s)
});

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid valid commitment")]
    InvalidValueCommitment,
}

impl std::ops::Add<Commitment> for Commitment {
    type Output = Commitment;
    fn add(self, rhs: Commitment) -> Self::Output {
        Commitment(self.0 + rhs.0)
    }
}

impl std::ops::Sub<Commitment> for Commitment {
    type Output = Commitment;
    fn sub(self, rhs: Commitment) -> Self::Output {
        Commitment(self.0 - rhs.0)
    }
}

impl std::ops::Neg for Commitment {
    type Output = Commitment;
    fn neg(self) -> Self::Output {
        Commitment(-self.0)
    }
}

impl From<Commitment> for [u8; 32] {
    fn from(commitment: Commitment) -> [u8; 32] {
        commitment.0.vartime_compress().0
    }
}

impl TryFrom<[u8; 32]> for Commitment {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = decaf377::Encoding(bytes)
            .vartime_decompress()
            .map_err(|_| Error::InvalidValueCommitment)?;

        Ok(Commitment(inner))
    }
}

impl TryFrom<&[u8]> for Commitment {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes = slice[..]
            .try_into()
            .map_err(|_| Error::InvalidValueCommitment)?;

        let inner = decaf377::Encoding(bytes)
            .vartime_decompress()
            .map_err(|_| Error::InvalidValueCommitment)?;

        Ok(Commitment(inner))
    }
}

impl Protobuf<pb::ValueCommitment> for Commitment {}

impl From<Commitment> for pb::ValueCommitment {
    fn from(cv: Commitment) -> Self {
        Self {
            inner: cv.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::ValueCommitment> for Commitment {
    type Error = anyhow::Error;
    fn try_from(value: pb::ValueCommitment) -> Result<Self, Self::Error> {
        value.inner.as_slice().try_into().map_err(Into::into)
    }
}
