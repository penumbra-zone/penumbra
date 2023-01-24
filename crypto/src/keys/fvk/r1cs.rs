use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Element, FieldExt, Fq, Fr,
};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use once_cell::sync::Lazy;

pub(crate) static SPENDAUTH_BASEPOINT: Lazy<Element> = Lazy::new(decaf377::basepoint);

pub struct RandomizedVerificationKey {
    pub inner: ElementVar,
}

impl AllocVar<VerificationKey<SpendAuth>, Fq> for RandomizedVerificationKey {
    fn new_variable<T: std::borrow::Borrow<VerificationKey<SpendAuth>>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner1 = f()?;
        let inner: VerificationKey<SpendAuth> = *inner1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => {
                let point = decaf377::Encoding(*inner.as_ref())
                    .vartime_decompress()
                    .unwrap();
                let element_var: ElementVar =
                    AllocVar::<Element, Fq>::new_input(cs.clone(), || Ok(point))?;
                Ok(Self { inner: element_var })
            }
            AllocationMode::Witness => unimplemented!(),
        }
    }
}

impl RandomizedVerificationKey {
    pub fn compress_to_field(&self) -> Result<FqVar, SynthesisError> {
        self.inner.compress_to_field()
    }
}

impl EqGadget<Fq> for RandomizedVerificationKey {
    fn is_eq(&self, other: &Self) -> Result<Boolean<Fq>, SynthesisError> {
        let self_fq = self.inner.compress_to_field()?;
        let other_fq = other.compress_to_field()?;
        self_fq.is_eq(&other_fq)
    }
}

pub struct AuthorizationKeyVar {
    pub inner: ElementVar,
}

impl AllocVar<VerificationKey<SpendAuth>, Fq> for AuthorizationKeyVar {
    fn new_variable<T: std::borrow::Borrow<VerificationKey<SpendAuth>>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner1 = f()?;
        let inner: VerificationKey<SpendAuth> = *inner1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let ak_point = decaf377::Encoding(*inner.as_ref())
                    .vartime_decompress()
                    .unwrap();
                let ak_element_var: ElementVar =
                    AllocVar::<Element, Fq>::new_witness(cs.clone(), || Ok(ak_point))?;
                Ok(Self {
                    inner: ak_element_var,
                })
            }
        }
    }
}

impl AuthorizationKeyVar {
    pub fn randomize(
        &self,
        spend_auth_randomizer: &SpendAuthRandomizerVar,
    ) -> Result<RandomizedVerificationKey, SynthesisError> {
        let cs = self.inner.cs();
        let spend_auth_basepoint_var = ElementVar::new_constant(cs.clone(), *SPENDAUTH_BASEPOINT)?;
        let point = self.inner.clone()
            + spend_auth_basepoint_var
                .scalar_mul_le(spend_auth_randomizer.inner.to_bits_le()?.iter())?;
        Ok(RandomizedVerificationKey { inner: point })
    }
}

pub struct SpendAuthRandomizerVar {
    inner: Vec<UInt8<Fq>>,
}

impl AllocVar<Fr, Fq> for SpendAuthRandomizerVar {
    fn new_variable<T: std::borrow::Borrow<Fr>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let inner1 = f()?;
        let inner: Fr = *inner1.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let spend_auth_randomizer_arr: [u8; 32] = inner.to_bytes();
                Ok(Self {
                    inner: UInt8::new_witness_vec(cs.clone(), &spend_auth_randomizer_arr)?,
                })
            }
        }
    }
}
