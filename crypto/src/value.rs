//! Values (?)

use std::{
    convert::{TryFrom, TryInto},
    ops::Deref,
};

use ark_ff::PrimeField;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror;

use crate::{asset, Fq, Fr};

#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Value {
    pub amount: u64,
    // The asset ID. 256 bits.
    #[serde(with = "serde_with::rust::display_fromstr")]
    pub asset_id: asset::Id,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Commitment(pub decaf377::Element);

pub static VALUE_BLINDING_GENERATOR: Lazy<decaf377::Element> = Lazy::new(|| {
    let s = Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"decaf377-rdsa-binding").as_bytes());
    decaf377::Element::map_to_group_cdh(&s)
});

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid valid commitment")]
    InvalidValueCommitment,
}

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

impl From<Commitment> for [u8; 32] {
    fn from(commitment: Commitment) -> [u8; 32] {
        commitment.0.compress().0
    }
}

impl TryFrom<[u8; 32]> for Commitment {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = decaf377::Encoding(bytes)
            .decompress()
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
            .decompress()
            .map_err(|_| Error::InvalidValueCommitment)?;

        Ok(Commitment(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_value_commitments() {
        use ark_ff::Field;

        let pen_denom = asset::REGISTRY.parse_base("upenumbra").unwrap();
        let atom_denom = asset::REGISTRY
            .parse_base("HubPort/HubChannel/uatom")
            .unwrap();

        let pen_id = asset::Id::from(pen_denom);
        let atom_id = asset::Id::from(atom_denom);

        // some values of different types
        let v1 = Value {
            amount: 10,
            asset_id: pen_id,
        };
        let v2 = Value {
            amount: 8,
            asset_id: pen_id,
        };
        let v3 = Value {
            amount: 2,
            asset_id: pen_id,
        };
        let v4 = Value {
            amount: 13,
            asset_id: atom_id,
        };
        let v5 = Value {
            amount: 17,
            asset_id: atom_id,
        };
        let v6 = Value {
            amount: 30,
            asset_id: atom_id,
        };

        // some random-looking blinding factors
        let b1 = Fr::from(-129).inverse().unwrap();
        let b2 = Fr::from(-199).inverse().unwrap();
        let b3 = Fr::from(-121).inverse().unwrap();
        let b4 = Fr::from(-179).inverse().unwrap();
        let b5 = Fr::from(-379).inverse().unwrap();
        let b6 = Fr::from(-879).inverse().unwrap();

        // form commitments
        let c1 = v1.commit(b1);
        let c2 = v2.commit(b2);
        let c3 = v3.commit(b3);
        let c4 = v4.commit(b4);
        let c5 = v5.commit(b5);
        let c6 = v6.commit(b6);

        // values sum to 0, so this is a commitment to 0...
        let c0 = c1 - c2 - c3 + c4 + c5 - c6;
        // with the following synthetic blinding factor:
        let b0 = b1 - b2 - b3 + b4 + b5 - b6;

        // so c0 = 0 * G_v1 + 0 * G_v2 + b0 * H
        assert_eq!(c0.0, b0 * VALUE_BLINDING_GENERATOR.deref());
    }
}
