use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use decaf377::{Fq, Fr};

use crate::addresses::PaymentAddress;
use crate::keys::EphemeralPublicKey;
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

        // TODO: This interface will change once we instantiate a key agreement scheme
        // on decaf377 similar to sec 5.4.5.3 in the ZCash spec.
        let ephemeral_key = EphemeralPublicKey::new();

        Self {
            value_commitment,
            note_commitment,
            ephemeral_key,
        }
    }
}
