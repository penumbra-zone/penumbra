use decaf377::Fq;

use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::r1cs::FqVar;
use penumbra_sdk_proto::crypto::decaf377_rdsa::v1::NullifierDerivingKey;

pub const NK_LEN_BYTES: usize = 32;

/// Allows deriving the nullifier associated with a positioned piece of state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NullifierKey(pub Fq);

impl NullifierKey {
    /// Returns the byte encoding of the nullifier key.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

// Simple byte conversions following VerificationKey pattern
impl From<NullifierKey> for [u8; 32] {
    fn from(nk: NullifierKey) -> [u8; 32] {
        nk.to_bytes()
    }
}

impl From<&NullifierKey> for [u8; 32] {
    fn from(nk: &NullifierKey) -> [u8; 32] {
        nk.to_bytes()
    }
}

impl TryFrom<[u8; 32]> for NullifierKey {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        let fq = Fq::from_bytes_checked(&bytes)
            .map_err(|_| anyhow::anyhow!("Invalid field element for NullifierKey"))?;
        Ok(NullifierKey(fq))
    }
}

impl TryFrom<&[u8]> for NullifierKey {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "NullifierKey must be exactly 32 bytes, got {}",
                bytes.len()
            ));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        array.try_into()
    }
}

impl TryFrom<Vec<u8>> for NullifierKey {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        bytes.as_slice().try_into()
    }
}

impl From<NullifierKey> for NullifierDerivingKey {
    fn from(key: NullifierKey) -> Self {
        Self {
            inner: key.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<NullifierDerivingKey> for NullifierKey {
    type Error = anyhow::Error;

    fn try_from(value: NullifierDerivingKey) -> Result<Self, Self::Error> {
        value.inner.as_slice().try_into()
    }
}

/// Represents the `NullifierKey` as a variable in an R1CS constraint system.
pub struct NullifierKeyVar {
    pub inner: FqVar,
}

impl AllocVar<NullifierKey, Fq> for NullifierKeyVar {
    fn new_variable<T: std::borrow::Borrow<NullifierKey>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner: NullifierKey = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => Ok(Self {
                inner: FqVar::new_witness(cs, || Ok(inner.0))?,
            }),
        }
    }
}

impl R1CSVar<Fq> for NullifierKeyVar {
    type Value = NullifierKey;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let inner_fq = self.inner.value()?;
        Ok(NullifierKey(inner_fq))
    }
}
