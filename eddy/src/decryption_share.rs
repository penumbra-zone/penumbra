use rand_core::{CryptoRng, RngCore};

use crate::{limb, Ciphertext, PrivateKeyShare, PublicKeyShare, TranscriptProtocol};

#[derive(Debug, Clone)]
pub enum Verified {}
#[derive(Debug, Clone)]
pub enum Unverified {}

pub trait VerificationState {}
impl VerificationState for Verified {}
impl VerificationState for Unverified {}

/// A share of a decryption of a particular [`Ciphertext`].
#[derive(Debug, Clone)]
pub struct DecryptionShare<S: VerificationState> {
    pub(crate) participant_index: u32,
    pub(crate) share0: limb::DecryptionShare<S>,
    pub(crate) share1: limb::DecryptionShare<S>,
    pub(crate) share2: limb::DecryptionShare<S>,
    pub(crate) share3: limb::DecryptionShare<S>,

    marker: std::marker::PhantomData<S>,
}

impl PrivateKeyShare {
    #[allow(non_snake_case)]
    pub fn decryption_share<R: RngCore + CryptoRng>(
        &self,
        ciphertext: &Ciphertext,
        transcript: &mut merlin::Transcript,
        mut rng: R,
    ) -> DecryptionShare<Unverified> {
        transcript.begin_decryption();
        transcript.append_public_key_share(&self.cached_pub);

        let share0 = self.limb_decryption_share(&ciphertext.c0, transcript, &mut rng);
        let share1 = self.limb_decryption_share(&ciphertext.c1, transcript, &mut rng);
        let share2 = self.limb_decryption_share(&ciphertext.c2, transcript, &mut rng);
        let share3 = self.limb_decryption_share(&ciphertext.c3, transcript, &mut rng);

        DecryptionShare::<Unverified> {
            participant_index: self.index,
            share0,
            share1,
            share2,
            share3,
            marker: std::marker::PhantomData,
        }
    }
}

impl DecryptionShare<Unverified> {
    #[allow(non_snake_case)]
    pub fn verify(
        &self,
        ctxt: &Ciphertext,
        pub_key_share: &PublicKeyShare,
        transcript: &mut merlin::Transcript,
    ) -> anyhow::Result<DecryptionShare<Verified>> {
        // This check isn't essential for security, because if we have the wrong
        // key share, the transcript won't match anyways, but it's a helpful
        // check against misuse.
        if self.participant_index != pub_key_share.index {
            return Err(anyhow::anyhow!(
                "decryption share participant index {} does not match public key share index {}",
                self.participant_index,
                pub_key_share.index
            ));
        }

        transcript.begin_decryption();
        transcript.append_public_key_share(pub_key_share);

        let share0 = self.share0.verify(&ctxt.c0, pub_key_share, transcript)?;
        let share1 = self.share1.verify(&ctxt.c1, pub_key_share, transcript)?;
        let share2 = self.share2.verify(&ctxt.c2, pub_key_share, transcript)?;
        let share3 = self.share3.verify(&ctxt.c3, pub_key_share, transcript)?;

        Ok(DecryptionShare::<Verified> {
            participant_index: self.participant_index,
            share0,
            share1,
            share2,
            share3,

            marker: std::marker::PhantomData,
        })
    }
}
