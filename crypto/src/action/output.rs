use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use crate::{addresses::PaymentAddress, ka, memo::MemoPlaintext, note, value, Fq, Fr, Note, Value};

pub const OVK_WRAPPED_LEN_BYTES: usize = 80;

pub struct Output {
    pub body: Body,
    //pub encrypted_memo: MemoCiphertext,
    pub memo: MemoPlaintext,
    // Below field is unusued until we implement note encryption.
    // pub ovk_wrapped_key: [u8; OVK_WRAPPED_LEN_BYTES],
}

pub struct Body {
    // Value commitment.
    pub value_commitment: value::Commitment,
    // Note commitment.
    pub note_commitment: note::Commitment,
    pub ephemeral_key: ka::Public,
    // TODO: Encrypted note
    // TODO: Proof
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        value: Value,
        v_blinding: Fr,
        dest: &PaymentAddress,
    ) -> Body {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
        let value_commitment = value.commit(v_blinding);

        let note_blinding = Fq::rand(rng);

        let note = Note::new(dest, value, note_blinding);
        let note_commitment = note.commit();
        // TODO: Encrypt note here and add to a field in the Body struct (later).

        let esk = ka::Secret::new(rng);
        let ephemeral_key = esk.diversified_public(note.dest.diversified_generator());

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
        }
    }
}
