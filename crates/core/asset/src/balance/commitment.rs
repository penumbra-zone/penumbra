use std::ops::Deref;

use ark_r1cs_std::prelude::*;
use ark_r1cs_std::uint8::UInt8;
use ark_relations::r1cs::SynthesisError;
use decaf377::r1cs::ElementVar;
use decaf377::Fq;
use decaf377::Fr;
use once_cell::sync::Lazy;
use penumbra_proto::penumbra::core::asset::v1 as pb;
use penumbra_proto::DomainType;

use crate::value::ValueVar;
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

impl ValueVar {
    pub fn commit(
        &self,
        value_blinding: Vec<UInt8<Fq>>,
    ) -> Result<BalanceCommitmentVar, SynthesisError> {
        let cs = self.amount().cs();
        let value_blinding_generator = ElementVar::new_constant(cs, *VALUE_BLINDING_GENERATOR)?;

        let asset_generator = self.asset_id.value_generator()?;
        let value_amount = self.amount();
        let commitment = asset_generator.scalar_mul_le(value_amount.to_bits_le()?.iter())?
            + value_blinding_generator.scalar_mul_le(value_blinding.to_bits_le()?.iter())?;

        Ok(BalanceCommitmentVar { inner: commitment })
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

pub struct BalanceCommitmentVar {
    pub inner: ElementVar,
}

impl AllocVar<Commitment, Fq> for BalanceCommitmentVar {
    fn new_variable<T: std::borrow::Borrow<Commitment>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: Commitment = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => {
                let element_var: ElementVar = AllocVar::new_input(cs, || Ok(inner.0))?;
                Ok(Self { inner: element_var })
            }
            AllocationMode::Witness => unimplemented!(),
        }
    }
}

impl R1CSVar<Fq> for BalanceCommitmentVar {
    type Value = Commitment;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner = self.inner.value()?;
        Ok(Commitment(inner))
    }
}

impl std::ops::Add<BalanceCommitmentVar> for BalanceCommitmentVar {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner + rhs.inner,
        }
    }
}

impl std::ops::Sub<BalanceCommitmentVar> for BalanceCommitmentVar {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner - rhs.inner,
        }
    }
}

impl EqGadget<Fq> for BalanceCommitmentVar {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        self.inner.is_eq(&other.inner)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid valid commitment")]
    InvalidBalanceCommitment,
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
            .map_err(|_| Error::InvalidBalanceCommitment)?;

        Ok(Commitment(inner))
    }
}

impl TryFrom<&[u8]> for Commitment {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes = slice[..]
            .try_into()
            .map_err(|_| Error::InvalidBalanceCommitment)?;

        let inner = decaf377::Encoding(bytes)
            .vartime_decompress()
            .map_err(|_| Error::InvalidBalanceCommitment)?;

        Ok(Commitment(inner))
    }
}

impl DomainType for Commitment {
    type Proto = pb::BalanceCommitment;
}

impl From<Commitment> for pb::BalanceCommitment {
    fn from(cv: Commitment) -> Self {
        Self {
            inner: cv.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::BalanceCommitment> for Commitment {
    type Error = anyhow::Error;
    fn try_from(value: pb::BalanceCommitment) -> Result<Self, Self::Error> {
        value.inner.as_slice().try_into().map_err(Into::into)
    }
}
