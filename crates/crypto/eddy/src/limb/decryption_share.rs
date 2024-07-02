use decaf377::Fr;
use rand_core::{CryptoRng, RngCore};

use super::Ciphertext;
use crate::{
    decryption_share::{Unverified, VerificationStatus, Verified},
    PrivateKeyShare, PublicKeyShare, TranscriptProtocol,
};

/// Threshold decryption share of a given encrypted value.
#[derive(Debug, Clone)]
pub struct DecryptionShare<S: VerificationStatus> {
    pub(crate) decryption_share: decaf377::Element,
    proof: DecryptionShareProof,
    pub(crate) participant_index: u32, // used for threshold decryption

    _marker: std::marker::PhantomData<S>,
}

#[derive(Debug, Clone)]
struct DecryptionShareProof {
    /// The challenge scalar
    c: decaf377::Fr,
    /// The response to the challenge
    r: decaf377::Fr,
}

impl PrivateKeyShare {
    #[allow(non_snake_case)]
    pub(crate) fn limb_decryption_share<R: RngCore + CryptoRng>(
        &self,
        ciphertext: &Ciphertext,
        transcript: &mut merlin::Transcript,
        mut rng: R,
    ) -> DecryptionShare<Unverified> {
        // compute the decryption share (self.key_share * ciphertext.c1)
        let decryption_share = self.key_share * ciphertext.c1;

        // compute the proof that decryption_share was correctly created using
        // the key share from dkg and the ciphertext.

        // Start feeding public data into the transcript
        transcript.begin_limb_decryption();
        // This is already included in the beginning of value decryption
        // transcript.append_public_key_share(&self.cached_pub);
        transcript.append_limb_ciphertext(ciphertext);
        transcript.append_decryption_share_point(&decryption_share);

        // First, generate a blinding factor and commit to it.
        let k = Fr::rand(
            // Use the Merlin transcript RNG to generate the blinding factor
            // This ensures the randomness is bound to:
            // - the entire public context (above)
            // - our secret key share
            // - fresh randomness from the provided RNG.
            &mut transcript
                .build_rng()
                .rekey_with_witness_bytes(b"key_share", &self.key_share.to_bytes())
                .finalize(&mut rng),
        );

        // We need one commitment for each LHS of the proof statement.
        let kB = k * decaf377::Element::GENERATOR;
        let kC_1 = k * ciphertext.c1;

        // Now append the commitments to the transcript...
        transcript.append_blinding_commitment(b"kB", &kB);
        transcript.append_blinding_commitment(b"kC_1", &kC_1);

        // ... and finally generate the challenge scalar to compute the response.
        let challenge = transcript.challenge_scalar(b"c");
        let response = k - self.key_share * challenge;

        DecryptionShare::<Unverified> {
            decryption_share,
            proof: DecryptionShareProof {
                c: challenge,
                r: response,
            },
            participant_index: self.participant_index,
            _marker: std::marker::PhantomData,
        }
    }
}

impl DecryptionShare<Unverified> {
    #[allow(non_snake_case)]
    pub fn verify(
        &self,
        ciphertext: &Ciphertext,
        pub_key_share: &PublicKeyShare,
        transcript: &mut merlin::Transcript,
    ) -> anyhow::Result<DecryptionShare<Verified>> {
        let kB = decaf377::Element::GENERATOR * self.proof.r
            + pub_key_share.pub_key_share * self.proof.c;
        let kC_1 = ciphertext.c1 * self.proof.r + self.decryption_share * self.proof.c;

        transcript.begin_limb_decryption();
        // We bind to the public key before beginning individual limbs.
        // transcript.append_public_key_share(pub_key_share);
        transcript.append_limb_ciphertext(ciphertext);
        transcript.append_decryption_share_point(&self.decryption_share);
        transcript.append_blinding_commitment(b"kB", &kB);
        transcript.append_blinding_commitment(b"kC_1", &kC_1);

        let challenge = transcript.challenge_scalar(b"c");

        if self.proof.c == challenge {
            Ok(DecryptionShare::<Verified> {
                decryption_share: self.decryption_share,
                participant_index: pub_key_share.participant_index,
                proof: self.proof.clone(),
                _marker: std::marker::PhantomData,
            })
        } else {
            Err(anyhow::anyhow!(
                "Recomputed challenge {:?} did not match expected challenge {:?}",
                challenge,
                self.proof.c
            ))
        }
    }
}
