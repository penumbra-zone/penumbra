use ark_ff::PrimeField;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use std::convert::{TryFrom, TryInto};
use thiserror;

use crate::{ka, Fq, Value};

// TODO: Should have a `leadByte` as in Sapling and Orchard note plaintexts?
// Do we need that in addition to the tx version?

/// A plaintext Penumbra note.
pub struct Note {
    // Value (32-byte asset ID plus 32-byte amount). 64 bytes.
    pub value: Value,

    // Commitment trapdoor. 32 bytes.
    pub note_blinding: Fq,

    // The diversified base and transmission key of the destination address.
    pub diversified_generator: decaf377::Element,
    pub transmission_key: ka::Public,
}

/// The domain separator used to generate note commitments.
static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid note commitment")]
    InvalidNoteCommitment,
}

impl Note {
    pub fn new(
        diversified_generator: &decaf377::Element,
        transmission_key: &ka::Public,
        value: Value,
        note_blinding: Fq,
    ) -> Self {
        Note {
            value: value,
            note_blinding: note_blinding,
            diversified_generator: diversified_generator.clone(),
            transmission_key: transmission_key.clone(),
        }
    }

    pub fn commit(&self) -> Result<Commitment, Error> {
        let commit = poseidon377::hash_5(
            &NOTECOMMIT_DOMAIN_SEP,
            (
                self.note_blinding,
                self.value.amount.into(),
                self.value.asset_id.0,
                self.diversified_generator.compress_to_field(),
                Fq::from_bytes(self.transmission_key.0)
                    .map_err(|_| Error::InvalidNoteCommitment)?,
            ),
        );

        Ok(Commitment(commit))
    }
}

// Note commitment.
#[derive(Clone, Copy, PartialEq)]
pub struct Commitment(pub Fq);

impl Into<[u8; 32]> for Commitment {
    fn into(self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

impl TryFrom<[u8; 32]> for Commitment {
    type Error = Error;

    fn try_from(bytes: [u8; 32]) -> Result<Commitment, Self::Error> {
        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}

impl TryFrom<&[u8]> for Commitment {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Commitment, Self::Error> {
        let bytes: [u8; 32] = slice[..]
            .try_into()
            .map_err(|_| Error::InvalidNoteCommitment)?;

        let inner = Fq::from_bytes(bytes).map_err(|_| Error::InvalidNoteCommitment)?;

        Ok(Commitment(inner))
    }
}
