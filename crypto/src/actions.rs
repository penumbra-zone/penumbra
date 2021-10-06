use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use decaf377::{Fq, Fr};
use decaf377_rdsa;

use crate::addresses::PaymentAddress;
use crate::keys::{EphemeralPublicKey, SpendAuthorizationKey};
use crate::note::{Note, NoteCommitment};
use crate::nullifier::Nullifier;
use crate::value::{Commitment, Value};

pub struct OutputDescriptionBody {
    // Value commitment.
    pub cv: Commitment,
    // Note commitment.
    pub cmu: NoteCommitment,
    pub ephemeral_key: EphemeralPublicKey,
    // TODO: Encrypted note
    // TODO: Proof
}

impl OutputDescriptionBody {
    pub fn new<R: RngCore + CryptoRng>(
        mut rng: R,
        value: Value,
        dest: PaymentAddress,
    ) -> OutputDescriptionBody {
        // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
        let v_blinding = Fr::rand(&mut rng);
        let cv = value.commit(v_blinding);

        let note_blinding = Fq::rand(&mut rng);
        let cm = NoteCommitment::new(&dest, &value, &note_blinding);

        let _note = Note::new(&dest, &value, &note_blinding);
        // TODO: Encrypt note here and add to a field in the OutputDescriptionBody struct (later).

        // TODO: This interface will change once we instantiate a key agreement scheme
        // on decaf377 similar to sec 5.4.5.3 in the ZCash spec.
        let ephemeral_key = EphemeralPublicKey::new();

        Self {
            cv,
            cmu: cm,
            ephemeral_key,
        }
    }
}

pub struct SpendDescriptionBody {
    // Value commitment.
    pub cv: Commitment,
    pub nullifier: Nullifier,
    // Randomized verification key.
    pub rk: decaf377_rdsa::VerificationKey<decaf377_rdsa::SpendAuth>,
    // TODO: Proof
}

impl SpendDescriptionBody {
    pub fn new<R: RngCore + CryptoRng>(
        mut rng: R,
        cv: Commitment,
        nullifier: Nullifier,
        ask: SpendAuthorizationKey, // wrong key?
    ) -> SpendDescriptionBody {
        let a = Fr::rand(&mut rng);
        let rk = ask.randomize(a);
        SpendDescriptionBody { cv, nullifier, rk }
    }
}
