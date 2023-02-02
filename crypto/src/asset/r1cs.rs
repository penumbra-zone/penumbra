use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::SynthesisError;
use decaf377::{
    r1cs::{ElementVar, FqVar},
    Fq,
};

use super::VALUE_GENERATOR_DOMAIN_SEP;

#[derive(Clone)]
pub struct AssetIdVar {
    pub asset_id: FqVar,
}

impl AllocVar<crate::asset::Id, Fq> for AssetIdVar {
    fn new_variable<T: std::borrow::Borrow<crate::asset::Id>>(
        cs: impl Into<ark_relations::r1cs::Namespace<Fq>>,
        f: impl FnOnce() -> Result<T, SynthesisError>,
        mode: ark_r1cs_std::prelude::AllocationMode,
    ) -> Result<Self, SynthesisError> {
        let ns = cs.into();
        let cs = ns.cs();
        let asset_id: crate::asset::Id = *f()?.borrow();
        match mode {
            AllocationMode::Constant => unimplemented!(),
            AllocationMode::Input => unimplemented!(),
            AllocationMode::Witness => {
                let inner_asset_id_var = FqVar::new_witness(cs, || Ok(asset_id.0))?;
                Ok(Self {
                    asset_id: inner_asset_id_var,
                })
            }
        }
    }
}

impl AssetIdVar {
    pub fn value_generator(&self) -> Result<ElementVar, SynthesisError> {
        let cs = self.asset_id.cs();
        let value_generator_domain_sep =
            FqVar::new_constant(cs.clone(), *VALUE_GENERATOR_DOMAIN_SEP)?;
        let hashed_asset_id =
            poseidon377::r1cs::hash_1(cs, &value_generator_domain_sep, self.asset_id.clone())?;
        ElementVar::encode_to_curve(&hashed_asset_id)
    }
}
