use std::ops::{Add, AddAssign};

use crate::decryption_share::Verified;
use crate::limb::DecryptionShare;
use ark_ff::One;

/// an Elgamal ciphertext (c1, c2).
#[derive(Default, Debug, Clone, Copy)]
pub struct Ciphertext {
    pub(crate) c1: decaf377::Element,
    pub(crate) c2: decaf377::Element,
}

// compute the lagrange coefficient for the participant given by `participant_index` in the set of
// participants given by participant_indices
fn lagrange_coefficient(participant_index: u32, participant_indices: &[u32]) -> decaf377::Fr {
    participant_indices
        .iter()
        .filter(|x| **x != participant_index)
        .fold(decaf377::Fr::one(), |acc, x| {
            let n = decaf377::Fr::from(*x);
            let i = decaf377::Fr::from(participant_index);

            acc * (n / (n - i))
        })
}

impl Ciphertext {
    pub fn decrypt(&self, shares: Vec<&DecryptionShare<Verified>>) -> decaf377::Element {
        let indices = shares
            .iter()
            .map(|s| s.participant_index)
            .collect::<Vec<_>>();

        let mut d = decaf377::Element::default();
        for share in shares {
            d += share.decryption_share * lagrange_coefficient(share.participant_index, &indices);
        }

        -d + self.c2
    }
}

impl Add<&Ciphertext> for &Ciphertext {
    type Output = Ciphertext;
    fn add(self, rhs: &Ciphertext) -> Self::Output {
        Ciphertext {
            c1: self.c1 + rhs.c1,
            c2: self.c2 + rhs.c2,
        }
    }
}

impl AddAssign<&Ciphertext> for Ciphertext {
    fn add_assign(&mut self, rhs: &Ciphertext) {
        self.c1 += rhs.c1;
        self.c2 += rhs.c2;
    }
}
