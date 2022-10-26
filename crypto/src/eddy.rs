use ark_ff::fields::PrimeField;
use ark_ff::One;
use ark_std::UniformRand;
use decaf377::Fr;
use std::collections::BTreeMap;

// a proof of correctness (r, s, t) for a given value encryption for a given public key (see the
// threshold cryptography spec for more details.)
#[derive(Copy, Clone)]
pub struct EncryptionProof {
    r: decaf377::Fr,
    s: decaf377::Fr,
    t: decaf377::Fr,
}

// an Elgamal ciphertext (c1, c2) along with the proof that the ciphertext is encrypted for a
// specific public key (presumably, the DKG public key).
#[derive(Clone, Copy)]
pub struct EncryptedValue {
    c1: decaf377::Element,
    c2: decaf377::Element,
    proof: Option<EncryptionProof>,
}

impl EncryptedValue {
    /// Verifies the [`EncryptionProof`] for this [`EncryptedValue`].
    ///
    /// See the [spec](https://protocol.penumbra.zone/main/crypto/flow-encryption/threshold-encryption.html) for more details.
    pub fn verify(&self, for_pubkey: decaf377::Element) -> Result<(), anyhow::Error> {
        let proof = self.proof.ok_or_else(|| anyhow::anyhow!("no proof"))?;
        let alpha = decaf377::basepoint() * proof.r + self.c1 * proof.t;
        let gamma = for_pubkey * proof.r + decaf377::basepoint() * proof.s + self.c2 * proof.t;
        let res_hash = blake2b_simd::Params::default()
            .personal(b"elgenc")
            .to_state()
            .update(&self.c1.vartime_compress().0)
            .update(&self.c2.vartime_compress().0)
            .update(&for_pubkey.vartime_compress().0)
            .update(&alpha.vartime_compress().0)
            .update(&gamma.vartime_compress().0)
            .finalize();
        let res = Fr::from_le_bytes_mod_order(res_hash.as_bytes());

        if res != proof.t {
            return Err(anyhow::anyhow!("invalid encryption proof"));
        }
        Ok(())
    }

    /// Add this [`EncryptedValue`] to another [`EncryptedValue`] to produce a new [`EncryptedValue`].
    /// NOTE: proofs are not aggregatable, so any aggregations must be verified independently.
    pub fn add(&self, other: &EncryptedValue) -> EncryptedValue {
        EncryptedValue {
            c1: self.c1 + other.c1,
            c2: self.c2 + other.c2,
            proof: None,
        }
    }
}

/// Encrypt the given `values` as value*decaf377::basepoint using the elgamal scheme, and compute
/// an [`EncrytionProof`] of correctness.
pub fn encrypt_value(value: decaf377::Fr, for_pubkey: decaf377::Element) -> EncryptedValue {
    let mut rng = rand::thread_rng();
    let e = Fr::rand(&mut rng);
    let c1 = e * decaf377::basepoint();
    let c2 = e * for_pubkey + value * decaf377::basepoint();

    let k1 = Fr::rand(&mut rng);
    let k2 = Fr::rand(&mut rng);
    let alpha = decaf377::basepoint() * k1;
    let gamma = for_pubkey * k1 + decaf377::basepoint() * k2;
    let challenge_hash = blake2b_simd::Params::default()
        .personal(b"elgenc")
        .to_state()
        .update(&c1.vartime_compress().0)
        .update(&c2.vartime_compress().0)
        .update(&for_pubkey.vartime_compress().0)
        .update(&alpha.vartime_compress().0)
        .update(&gamma.vartime_compress().0)
        .finalize();
    let t = Fr::from_le_bytes_mod_order(challenge_hash.as_bytes());
    let r = k1 - e * t;
    let s = k2 - value * t;

    let proof = EncryptionProof { r, s, t };
    EncryptedValue {
        c1,
        c2,
        proof: Some(proof),
    }
}

/// Decrypt a given [`EncryptedValue`] in the threshold setting using a set of decryption shares.
pub fn decrypt_value(
    encrypted_value: &EncryptedValue,
    decryption_shares: &[DecryptionShare],
    participant_commitments: &[decaf377::Element],
) -> Result<decaf377::Element, anyhow::Error> {
    for (share, participant_commitment) in decryption_shares.iter().zip(participant_commitments) {
        share.verify(encrypted_value.c1, *participant_commitment)?;
    }

    let indices = decryption_shares
        .iter()
        .map(|share| share.participant_index)
        .collect::<Vec<_>>();

    let mut d = decaf377::Element::default();
    for share in decryption_shares {
        d += share.share * lagrange_coefficient(share.participant_index, &indices);
    }
    Ok(-d + encrypted_value.c2)
}

/// Sum a set of encrypted values together using the additive homomorphism.
pub fn aggregate_values(
    values: &[EncryptedValue],
    dkg_pubkey: decaf377::Element,
) -> Result<EncryptedValue, anyhow::Error> {
    let mut res = values.first().unwrap().clone();
    res.verify(dkg_pubkey)?;
    for value in values.iter().skip(1) {
        value.verify(dkg_pubkey)?;
        res = res.add(value);
    }
    Ok(res)
}

// compute a look-up-table for the discrete logarithm of the set of values [1, 2, ..., maxval]
#[allow(dead_code)]
fn compute_lut(maxval: u64) -> BTreeMap<[u8; 32], u64> {
    let mut res = BTreeMap::new();
    for i in 1..maxval {
        let ge = Fr::from(i) * decaf377::basepoint();
        res.insert(ge.vartime_compress().0, i);
    }
    res
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

/// A proof of a threshold decryption share (r, t)
/// [spec](https://protocol.penumbra.zone/main/crypto/flow-encryption/threshold-encryption.html)
/// for more details
#[derive(Clone)]
pub struct DecryptionProof {
    r: decaf377::Fr,
    t: decaf377::Fr,
}

/// Threshold decryption share of a given encrypted value, along with its proof and the index of
/// the participant that created the share.
#[derive(Clone)]
pub struct DecryptionShare {
    share: decaf377::Element,
    proof: DecryptionProof,
    participant_index: u32,
}

impl DecryptionShare {
    /// Creates a decryption share (and proof) for the given `c1` using the participant's key share
    /// `private_key`. see the
    /// [spec](https://protocol.penumbra.zone/main/crypto/flow-encryption/threshold-encryption.html)
    /// for more details.
    pub fn new(
        private_key: decaf377::Fr,
        c1: decaf377::Element,
        participant_index: u32,
        participant_commitment: decaf377::Element,
    ) -> DecryptionShare {
        let mut rng = rand::thread_rng();
        let spi = private_key * c1;

        // construct the nizk proof
        let k = Fr::rand(&mut rng);
        let alpha = k * decaf377::basepoint();
        let gamma = k * c1;

        let res_hash = blake2b_simd::Params::default()
            .personal(b"elgdec")
            .to_state()
            .update(&spi.vartime_compress().0)
            .update(&c1.vartime_compress().0)
            .update(&participant_index.to_le_bytes())
            .update(&participant_commitment.vartime_compress().0)
            .update(&alpha.vartime_compress().0)
            .update(&gamma.vartime_compress().0)
            .finalize();
        let t = Fr::from_le_bytes_mod_order(res_hash.as_bytes());
        let r = k - private_key * t;

        DecryptionShare {
            share: spi,
            proof: DecryptionProof { r, t },
            participant_index,
        }
    }
    /// Verifies a decryption share over `c1` using the commitment to the participant's secret
    /// share (output from DKG)
    pub fn verify(
        &self,
        c1: decaf377::Element,
        dkg_commitment: decaf377::Element,
    ) -> Result<(), anyhow::Error> {
        let alpha = decaf377::basepoint() * self.proof.r + dkg_commitment * self.proof.t;
        let gamma = c1 * self.proof.r + self.share * self.proof.t;
        let res_hash = blake2b_simd::Params::default()
            .personal(b"elgdec")
            .to_state()
            .update(&self.share.vartime_compress().0)
            .update(&c1.vartime_compress().0)
            .update(&self.participant_index.to_le_bytes())
            .update(&dkg_commitment.vartime_compress().0)
            .update(&alpha.vartime_compress().0)
            .update(&gamma.vartime_compress().0)
            .finalize();
        let res = Fr::from_le_bytes_mod_order(res_hash.as_bytes());

        if res != self.proof.t {
            return Err(anyhow::anyhow!("invalid decryption share"));
        }

        Ok(())
    }
}

#[allow(clippy::disallowed_types)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use ark_ff::UniformRand;
    use rand::prelude::IteratorRandom;

    #[test]
    fn test_basic_encrypt_decrypt() {
        let mut rng = rand::thread_rng();
        let privkey = decaf377::Fr::rand(&mut rng);
        let pubkey = privkey * decaf377::basepoint();
        let value_to_encrypt = decaf377::Fr::from(1000u64);

        let encrypted = encrypt_value(value_to_encrypt, pubkey);

        assert!(encrypted.verify(pubkey).is_ok());
        let alt_pubkey = decaf377::Fr::rand(&mut rng) * decaf377::basepoint();
        assert!(encrypted.verify(alt_pubkey).is_err());

        let decrypted = -privkey * encrypted.c1 + encrypted.c2;
        assert_eq!(
            (value_to_encrypt * decaf377::basepoint())
                .vartime_compress()
                .0,
            decrypted.vartime_compress().0
        );
    }
    #[test]
    fn test_homomorphism() {
        let mut rng = rand::thread_rng();
        let privkey = decaf377::Fr::rand(&mut rng);
        let pubkey = privkey * decaf377::basepoint();

        let lut = compute_lut(1000);
        let values = [
            decaf377::Fr::from(100u64),
            decaf377::Fr::from(200u64),
            decaf377::Fr::from(300u64),
        ];
        let mut encrypted_values = Vec::new();
        for value in values.iter() {
            encrypted_values.push(encrypt_value(*value, pubkey));
        }

        let encrypted_aggregate = aggregate_values(&encrypted_values, pubkey).unwrap();

        let decrypted = -privkey * encrypted_aggregate.c1 + encrypted_aggregate.c2;
        assert_eq!(lut.get(&decrypted.vartime_compress().0).unwrap(), &600u64);
    }
    #[test]
    fn test_threshold_decryption() {
        // do a dkg (using frost dkg for now), the do aggregation + threshold decryption
        let lut = compute_lut(1000);
        let t = 10;
        let n = 20;

        let mut participants = Vec::new();
        for i in 1..n + 1 {
            participants.push(frost377::keygen::Participant::new(i, t));
        }
        let mut round1_messages = Vec::new();
        for participant in participants.iter() {
            round1_messages.push(participant.round_one());
        }
        for participant in participants.iter_mut() {
            participant
                .verify_roundone(round1_messages.clone())
                .unwrap();
        }
        let other_participants = participants.clone();
        for participant in participants.iter_mut() {
            for participant_other in other_participants.iter() {
                if participant.index == participant_other.index {
                    continue;
                }
                let round2_message = participant_other.round_two(participant.index);

                participant
                    .verify_and_add_roundtwo_response(&round2_message)
                    .unwrap();
            }
        }

        let mut pubkey_commitments = Vec::new();
        let mut dkg_outputs = Vec::new();
        for participant in participants.iter() {
            let output = participant.finalize().unwrap();
            dkg_outputs.push(output.clone());
            pubkey_commitments.push(output.private_share * decaf377::basepoint());
        }

        // encrypt a few values for the dkg pubkey
        let encrypted_values = [
            encrypt_value(decaf377::Fr::from(100u64), dkg_outputs[0].group_public_key),
            encrypt_value(decaf377::Fr::from(200u64), dkg_outputs[0].group_public_key),
            encrypt_value(decaf377::Fr::from(300u64), dkg_outputs[0].group_public_key),
        ];
        let aggregate_value =
            aggregate_values(&encrypted_values, dkg_outputs[0].group_public_key).unwrap();

        // produce n decryption shares
        let mut decryption_shares = Vec::new();
        for i in 0..n {
            let share = &dkg_outputs[i as usize];
            decryption_shares.push(DecryptionShare::new(
                share.private_share,
                aggregate_value.c1,
                share.participant_index,
                pubkey_commitments[i as usize],
            ));
        }

        // try threshold decryption with t/n shares
        let decrypted_value = decrypt_value(
            &aggregate_value,
            &decryption_shares[..t as usize],
            &pubkey_commitments,
        );

        assert!(
            lut.get(&decrypted_value.unwrap().vartime_compress().0)
                .unwrap()
                == &600u64
        );

        // randomly select t/n shares
        let mut subset = Vec::new();
        let mut subset_pubkey_commitments = Vec::new();
        let mut seen_shares = HashMap::new();
        while subset.len() < t as usize {
            let (i, random_share) = decryption_shares
                .iter()
                .enumerate()
                .choose(&mut rand::thread_rng())
                .unwrap();
            seen_shares
                .entry(random_share.participant_index)
                .or_insert_with(|| {
                    subset.push(random_share.clone());
                    subset_pubkey_commitments.push(pubkey_commitments[i as usize]);
                    true
                });
        }

        // try threshold decryption with randomly selected t/n shares
        let decrypted_value_rand_subset =
            decrypt_value(&aggregate_value, &subset, &subset_pubkey_commitments);

        assert!(
            lut.get(&decrypted_value_rand_subset.unwrap().vartime_compress().0)
                .unwrap()
                == &600u64
        );
    }
}
