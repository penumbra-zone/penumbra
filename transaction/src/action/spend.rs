use std::convert::{TryFrom, TryInto};

use anyhow::Error;
use bytes::Bytes;
use penumbra_crypto::{
    keys, merkle,
    proofs::transparent::SpendProof,
    rdsa::{Signature, SigningKey, SpendAuth, VerificationKey},
    value, Fr, Note, Nullifier,
};
use penumbra_proto::{transaction, Protobuf};

#[derive(Clone, Debug)]
pub struct Spend {
    pub body: Body,
    pub auth_sig: Signature<SpendAuth>,
    pub proof: SpendProof,
}

impl Protobuf<transaction::Spend> for Spend {}

impl From<Spend> for transaction::Spend {
    fn from(msg: Spend) -> Self {
        let sig_bytes: [u8; 64] = msg.auth_sig.into();
        let proof: Vec<u8> = msg.proof.into();
        transaction::Spend {
            body: Some(msg.body.into()),
            auth_sig: Bytes::copy_from_slice(&sig_bytes),
            zkproof: proof.into(),
        }
    }
}

impl TryFrom<transaction::Spend> for Spend {
    type Error = Error;

    fn try_from(proto: transaction::Spend) -> anyhow::Result<Self, Self::Error> {
        let body = proto
            .body
            .ok_or(anyhow::anyhow!("spend body malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;

        let sig_bytes: [u8; 64] = proto.auth_sig[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;

        let proof = (proto.zkproof[..])
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;

        Ok(Spend {
            body,
            auth_sig: sig_bytes.into(),
            proof,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub value_commitment: value::Commitment,
    pub nullifier: Nullifier,
    pub rk: VerificationKey<SpendAuth>,
}

impl Body {
    pub fn new(
        value_commitment: value::Commitment,
        ask: SigningKey<SpendAuth>,
        spend_auth_randomizer: Fr,
        merkle_path: merkle::Path,
        note: Note,
        v_blinding: Fr,
        nk: keys::NullifierKey,
    ) -> (Body, SpendProof) {
        let rsk = ask.randomize(&spend_auth_randomizer);
        let rk = rsk.into();
        let note_commitment = note.commit();
        let position = merkle_path.0.clone();
        let proof = SpendProof {
            // XXX: the position field duplicates data from the merkle path
            // probably not worth fixing before we just make them snarks...
            position,
            merkle_path,
            g_d: note.diversified_generator(),
            pk_d: note.transmission_key(),
            value: note.value(),
            v_blinding,
            note_commitment,
            note_blinding: note.note_blinding(),
            spend_auth_randomizer,
            ak: ask.into(),
            nk,
        };
        (
            Body {
                value_commitment,
                nullifier: nk.derive_nullifier(position, &note_commitment),
                rk,
            },
            proof,
        )
    }
}

impl Protobuf<transaction::SpendBody> for Body {}

impl From<Body> for transaction::SpendBody {
    fn from(msg: Body) -> Self {
        let cv_bytes: [u8; 32] = msg.value_commitment.into();
        let nullifier_bytes: [u8; 32] = msg.nullifier.into();
        let rk_bytes: [u8; 32] = msg.rk.into();
        transaction::SpendBody {
            cv: Bytes::copy_from_slice(&cv_bytes),
            nullifier: Bytes::copy_from_slice(&nullifier_bytes),
            rk: Bytes::copy_from_slice(&rk_bytes),
        }
    }
}

impl TryFrom<transaction::SpendBody> for Body {
    type Error = Error;

    fn try_from(proto: transaction::SpendBody) -> anyhow::Result<Self, Self::Error> {
        let value_commitment: value::Commitment = (proto.cv[..])
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;

        let nullifier = (proto.nullifier[..])
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;

        let rk_bytes: [u8; 32] = (proto.rk[..])
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;
        let rk = rk_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("spend body malformed"))?;

        Ok(Body {
            value_commitment,
            nullifier,
            rk,
        })
    }
}
