//! Transparent proofs for `MVP1` of the Penumbra system.

use ark_serialize::CanonicalSerialize;
use decaf377::FieldExt;
use decaf377_rdsa::{SpendAuth, VerificationKey};
use std::convert::{TryFrom, TryInto};
use thiserror;

use penumbra_proto::{transparent_proofs, Protobuf};

use crate::{
    action::error::ProtoError, asset, ka, keys, merkle, note, value, Fq, Fr, Nullifier, Value,
};

pub const OUTPUT_PROOF_LEN_BYTES: usize = 192;
// xx check the spend proof len
pub const SPEND_PROOF_LEN_BYTES: usize = 192;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid spend auth randomizer")]
    InvalidSpendAuthRandomizer,
}

/// Transparent proof for spending existing notes.
///
/// This structure keeps track of the auxiliary (private) inputs.
pub struct SpendProof {
    // Path to the note being spent in the note commitment merkle tree.
    pub merkle_path: merkle::Path,
    // Position of the note being spent in the note commitment merkle tree.
    pub position: merkle::Position,
    // The diversified base for the address.
    pub g_d: decaf377::Element,
    // The transmission key for the address.
    pub pk_d: ka::Public,
    // The value of the note.
    pub value: Value,
    // The blinding factor used for generating the value commitment.
    pub v_blinding: Fr,
    // The note commitment.
    pub note_commitment: note::Commitment,
    // The blinding factor used for generating the note commitment.
    pub note_blinding: Fq,
    // The randomizer used for generating the randomized spend auth key.
    pub spend_auth_randomizer: Fr,
    // The spend authorization key.
    pub ak: VerificationKey<SpendAuth>,
    // The nullifier deriving key.
    pub nk: keys::NullifierKey,
}

impl SpendProof {
    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * the merkle root of the note commitment tree,
    /// * value commitment of the note to be spent,
    /// * nullifier of the note to be spent,
    /// * the randomized verification spend key,
    pub fn verify(
        anchor: merkle::Root,
        value_commitment: value::Commitment,
        nullifier: Nullifier,
        rk: VerificationKey<SpendAuth>,
    ) -> bool {
        todo!()
    }
}

/// Transparent proof for new note creation.
///
/// This structure keeps track of the auxiliary (private) inputs.
pub struct OutputProof {
    // The diversified base for the destination address.
    pub g_d: decaf377::Element,
    // The transmission key for the destination address.
    pub pk_d: ka::Public,
    // The value of the newly created note.
    pub value: Value,
    // The blinding factor used for generating the value commitment.
    pub v_blinding: Fr,
    // The blinding factor used for generating the note commitment.
    pub note_blinding: Fq,
    // The ephemeral secret key that corresponds to the public key.
    pub esk: ka::Secret,
}

impl OutputProof {
    /// Called to verify the proof using the provided public inputs.
    ///
    /// The public inputs are:
    /// * value commitment of the new note,
    /// * note commitment of the new note,
    /// * the ephemeral public key used to generate the new note.
    pub fn verify(
        value_commitment: value::Commitment,
        note_commitment: note::Commitment,
        epk: ka::Public,
    ) -> bool {
        todo!()
    }
}

// Conversions

impl Protobuf<transparent_proofs::SpendProof> for SpendProof {}

impl From<SpendProof> for transparent_proofs::SpendProof {
    fn from(msg: SpendProof) -> Self {
        let ak_bytes: [u8; 32] = msg.ak.into();
        let nk_bytes: [u8; 32] = msg.nk.0.to_bytes();
        transparent_proofs::SpendProof {
            merkle_path_field_0: msg.merkle_path.0 as u32,
            merkle_path_field_1: msg
                .merkle_path
                .1
                .into_iter()
                .map(|x| x.0.to_bytes().into())
                .collect(),
            position: msg.position.into(),
            g_d: msg.g_d.compress().0.to_vec(),
            pk_d: msg.pk_d.0.to_vec(),
            value_amount: msg.value.amount,
            value_asset_id: msg.value.asset_id.0.to_bytes().to_vec(),
            v_blinding: msg.v_blinding.to_bytes().to_vec(),
            note_commitment: msg.note_commitment.0.to_bytes().to_vec(),
            note_blinding: msg.note_blinding.to_bytes().to_vec(),
            spend_auth_randomizer: msg.spend_auth_randomizer.to_bytes().to_vec(),
            ak: ak_bytes.into(),
            nk: nk_bytes.into(),
        }
    }
}

impl TryFrom<transparent_proofs::SpendProof> for SpendProof {
    type Error = ProtoError;

    fn try_from(proto: transparent_proofs::SpendProof) -> anyhow::Result<Self, Self::Error> {
        let g_d_bytes: [u8; 32] = proto
            .g_d
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;
        let g_d_encoding = decaf377::Encoding(g_d_bytes);

        let v_blinding_bytes: [u8; 32] = proto.v_blinding[..]
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;

        let ak_bytes: [u8; 32] = (proto.ak[..])
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;
        let ak = ak_bytes
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;

        let mut merkle_path_vec = Vec::<note::Commitment>::new();
        for merkle_path_segment in proto.merkle_path_field_1 {
            merkle_path_vec.push(
                merkle_path_segment[..]
                    .try_into()
                    .map_err(|_| ProtoError::ProofMalformed)?,
            );
        }

        Ok(SpendProof {
            merkle_path: (proto.merkle_path_field_0 as usize, merkle_path_vec),
            position: (proto.position as usize).into(),
            g_d: g_d_encoding
                .decompress()
                .map_err(|_| ProtoError::ProofMalformed)?,
            pk_d: ka::Public(
                proto
                    .pk_d
                    .try_into()
                    .map_err(|_| ProtoError::ProofMalformed)?,
            ),
            value: Value {
                amount: proto.value_amount,
                asset_id: asset::Id(
                    Fq::from_bytes(
                        proto
                            .value_asset_id
                            .try_into()
                            .map_err(|_| ProtoError::ProofMalformed)?,
                    )
                    .map_err(|_| ProtoError::ProofMalformed)?,
                ),
            },
            v_blinding: Fr::from_bytes(v_blinding_bytes).map_err(|_| ProtoError::ProofMalformed)?,
            note_commitment: (proto.note_commitment[..])
                .try_into()
                .map_err(|_| ProtoError::ProofMalformed)?,
            note_blinding: Fq::from_bytes(
                proto.note_blinding[..]
                    .try_into()
                    .map_err(|_| ProtoError::ProofMalformed)?,
            )
            .map_err(|_| ProtoError::ProofMalformed)?,
            spend_auth_randomizer: Fr::from_bytes(
                proto.spend_auth_randomizer[..]
                    .try_into()
                    .map_err(|_| ProtoError::ProofMalformed)?,
            )
            .map_err(|_| ProtoError::ProofMalformed)?,
            ak,
            nk: keys::NullifierKey(
                Fq::from_bytes(
                    proto.nk[..]
                        .try_into()
                        .map_err(|_| ProtoError::ProofMalformed)?,
                )
                .map_err(|_| ProtoError::ProofMalformed)?,
            ),
        })
    }
}

impl Protobuf<transparent_proofs::OutputProof> for OutputProof {}

impl From<OutputProof> for transparent_proofs::OutputProof {
    fn from(msg: OutputProof) -> Self {
        transparent_proofs::OutputProof {
            g_d: msg.g_d.compress().0.to_vec(),
            pk_d: msg.pk_d.0.to_vec(),
            value_amount: msg.value.amount,
            value_asset_id: msg.value.asset_id.0.to_bytes().to_vec(),
            v_blinding: msg.v_blinding.to_bytes().to_vec(),
            note_blinding: msg.note_blinding.to_bytes().to_vec(),
            esk: msg.esk.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<transparent_proofs::OutputProof> for OutputProof {
    type Error = ProtoError;

    fn try_from(proto: transparent_proofs::OutputProof) -> anyhow::Result<Self, Self::Error> {
        let g_d_bytes: [u8; 32] = proto
            .g_d
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;
        let g_d_encoding = decaf377::Encoding(g_d_bytes);

        let v_blinding_bytes: [u8; 32] = proto.v_blinding[..]
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;

        let esk_bytes: [u8; 32] = proto.esk[..]
            .try_into()
            .map_err(|_| ProtoError::ProofMalformed)?;
        let esk = ka::Secret::new_from_field(
            Fr::from_bytes(esk_bytes).map_err(|_| ProtoError::ProofMalformed)?,
        );

        Ok(OutputProof {
            g_d: g_d_encoding
                .decompress()
                .map_err(|_| ProtoError::ProofMalformed)?,
            pk_d: ka::Public(
                proto
                    .pk_d
                    .try_into()
                    .map_err(|_| ProtoError::ProofMalformed)?,
            ),
            value: Value {
                amount: proto.value_amount,
                asset_id: asset::Id(
                    Fq::from_bytes(
                        proto
                            .value_asset_id
                            .try_into()
                            .map_err(|_| ProtoError::ProofMalformed)?,
                    )
                    .map_err(|_| ProtoError::ProofMalformed)?,
                ),
            },
            v_blinding: Fr::from_bytes(v_blinding_bytes).map_err(|_| ProtoError::ProofMalformed)?,
            note_blinding: Fq::from_bytes(
                proto.note_blinding[..]
                    .try_into()
                    .map_err(|_| ProtoError::ProofMalformed)?,
            )
            .map_err(|_| ProtoError::ProofMalformed)?,
            esk,
        })
    }
}

impl Into<[u8; SPEND_PROOF_LEN_BYTES]> for SpendProof {
    fn into(self) -> [u8; SPEND_PROOF_LEN_BYTES] {
        let mut spend_auth_randomizer_bytes = [0u8; 32];
        self.spend_auth_randomizer
            .serialize(&mut spend_auth_randomizer_bytes[..])
            .expect("serialization into array should be infallible");

        let mut bytes = [0u8; SPEND_PROOF_LEN_BYTES];
        bytes.copy_from_slice(&spend_auth_randomizer_bytes);

        // TODO: Merkle path serialization and add in here

        // When we put more stuff into this transparent spend proof, add here.
        bytes
    }
}

impl TryFrom<&[u8]> for SpendProof {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<SpendProof, Self::Error> {
        let mut spend_auth_randomizer_bytes = [0u8; 32];
        spend_auth_randomizer_bytes.copy_from_slice(&bytes[0..32]);

        let spend_auth_randomizer = Fr::from_bytes(spend_auth_randomizer_bytes)
            .map_err(|_| Error::InvalidSpendAuthRandomizer)?;

        // TODO! Merkle path serialization.
        Ok(SpendProof {
            spend_auth_randomizer,
            merkle_path: merkle::Path::default(),
        })
    }
}

impl Into<[u8; OUTPUT_PROOF_LEN_BYTES]> for OutputProof {
    fn into(self) -> [u8; OUTPUT_PROOF_LEN_BYTES] {
        let bytes = [0u8; OUTPUT_PROOF_LEN_BYTES];
        // When we put more stuff into this transparent output proof, add here.
        bytes
    }
}
