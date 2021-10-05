// `SpendDescription`

// `OutputDescription`

use ark_ff::UniformRand;
use rand_core::{CryptoRng, RngCore};

use decaf377::{Fq, Fr};

use crate::addresses::PaymentAddress;
use crate::note::{Note, NoteCommitment};
use crate::value::Value;

fn make_output<R: RngCore + CryptoRng>(
    mut rng: R,
    value: Value,
    dest: PaymentAddress,
) -> (Note, NoteCommitment) {
    let g_d = dest.diversifier.diversified_generator();

    // TODO: p. 43 Spec. Decide whether to do leadByte 0x01 method or 0x02 or other.
    let v_blinding = Fr::rand(&mut rng);
    let cv = value.commit(v_blinding);

    let note_blinding = Fq::rand(&mut rng);
    let cm = NoteCommitment::new(&dest, &value, &note_blinding);

    (
        Note {
            diversifier: dest.diversifier,
            value,
            note_blinding,
        },
        cm,
    )
}
