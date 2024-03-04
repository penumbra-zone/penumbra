use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Element, Fq, Fr,
};
use decaf377_rdsa::{SpendAuth, VerificationKey, VerificationKeyBytes};
use once_cell::sync::Lazy;

fn generator() -> Element {
    Element::GENERATOR
}

pub(crate) static SPENDAUTH_BASEPOINT: Lazy<Element> = Lazy::new(generator);

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
        let inner: VerificationKey<SpendAuth> = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => {
                let point = decaf377::Encoding(*inner.as_ref())
                    .vartime_decompress()
                    .map_err(|_| SynthesisError::MalformedVerifyingKey)?;
                let element_var: ElementVar = AllocVar::new_input(cs, || Ok(point))?;
                Ok(Self { inner: element_var })
            }
            AllocationMode::Witness => unimplemented!(),
        }
    }
}

impl R1CSVar<Fq> for RandomizedVerificationKey {
    type Value = VerificationKey<SpendAuth>;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let point = self.inner.value()?;
        let key_bytes = point.vartime_compress();
        let verification_key_bytes: VerificationKeyBytes<SpendAuth> = key_bytes.0.into();
        Ok(
            VerificationKey::<SpendAuth>::try_from(verification_key_bytes)
                .expect("should be able to convert from bytes"),
        )
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
        let inner: VerificationKey<SpendAuth> = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let ak_point = decaf377::Encoding(*inner.as_ref())
                    .vartime_decompress()
                    .map_err(|_| SynthesisError::MalformedVerifyingKey)?;
                let ak_element_var: ElementVar =
                    AllocVar::<Element, Fq>::new_witness(cs, || Ok(ak_point))?;
                Ok(Self {
                    inner: ak_element_var,
                })
            }
        }
    }
}

impl R1CSVar<Fq> for AuthorizationKeyVar {
    type Value = VerificationKey<SpendAuth>;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let point = self.inner.value()?;
        let key_bytes = point.vartime_compress();
        let verification_key_bytes: VerificationKeyBytes<SpendAuth> = key_bytes.0.into();
        Ok(
            VerificationKey::<SpendAuth>::try_from(verification_key_bytes)
                .expect("should be able to convert from bytes"),
        )
    }
}

impl AuthorizationKeyVar {
    pub fn randomize(
        &self,
        spend_auth_randomizer: &SpendAuthRandomizerVar,
    ) -> Result<RandomizedVerificationKey, SynthesisError> {
        let cs = self.inner.cs();
        let spend_auth_basepoint_var = ElementVar::new_constant(cs, *SPENDAUTH_BASEPOINT)?;
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
        let inner: Fr = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let spend_auth_randomizer_arr: [u8; 32] = inner.to_bytes();
                Ok(Self {
                    inner: UInt8::new_witness_vec(cs, &spend_auth_randomizer_arr)?,
                })
            }
        }
    }
}

impl R1CSVar<Fq> for SpendAuthRandomizerVar {
    type Value = Fr;

    fn cs(&self) -> ark_relations::r1cs::ConstraintSystemRef<Fq> {
        self.inner.cs()
    }

    fn value(&self) -> Result<Self::Value, SynthesisError> {
        let mut bytes = [0u8; 32];
        for (i, byte) in self.inner.iter().enumerate() {
            bytes[i] = byte.value()?;
        }
        Ok(Fr::from_bytes_checked(&bytes).expect("can convert bytes to Fr"))
    }
}
