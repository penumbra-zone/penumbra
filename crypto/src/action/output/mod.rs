use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use decaf377::{Fq, Fr};
use decaf377_ka::{EphemeralPublicKey, EphemeralSecretKey};

use crate::addresses::PaymentAddress;
use crate::note;
use crate::value;
use crate::{Note, Value};

pub struct Body {
    // Value commitment.
    pub value_commitment: value::Commitment,
    // Note commitment.
    pub note_commitment: note::Commitment,
    pub ephemeral_key: EphemeralPublicKey,
    // TODO: Encrypted note
    // TODO: Proof
}

impl Body {
    pub fn new<R: RngCore + CryptoRng>(mut rng: R, value: Value, dest: PaymentAddress) -> Body {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
        let v_blinding = Fr::rand(&mut rng);
        let value_commitment = value.commit(v_blinding);

        let note_blinding = Fq::rand(&mut rng);
        let note_commitment = note::Commitment::new(&dest, &value, &note_blinding);

        let _note = Note::new(&dest, &value, &note_blinding);
        // TODO: Encrypt note here and add to a field in the Body struct (later).

        let esk = EphemeralSecretKey::generate(&mut rng);
        let ephemeral_key = esk.derive_public();

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
        }
    }
}
