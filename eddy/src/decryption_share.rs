use rand_core::{CryptoRng, RngCore};

use crate::{limb, Ciphertext, PrivateKeyShare, PublicKeyShare, TranscriptProtocol};

/// A share of a decryption of a particular [`Ciphertext`].
///
/// TODO: use some kind of typestate to record verification state?
pub struct DecryptionShare {
    participant_index: u32,
    share0: limb::DecryptionShare,
    share1: limb::DecryptionShare,
    share2: limb::DecryptionShare,
    share3: limb::DecryptionShare,
}

impl PrivateKeyShare {
    #[allow(non_snake_case)]
    pub fn decryption_share<R: RngCore + CryptoRng>(
        &self,
        ciphertext: &Ciphertext,
        transcript: &mut merlin::Transcript,
        mut rng: R,
    ) -> DecryptionShare {
        transcript.begin_decryption();
        transcript.append_public_key_share(&self.cached_pub);

        let share0 = self.limb_decryption_share(&ciphertext.c0, transcript, &mut rng);
        let share1 = self.limb_decryption_share(&ciphertext.c1, transcript, &mut rng);
        let share2 = self.limb_decryption_share(&ciphertext.c2, transcript, &mut rng);
        let share3 = self.limb_decryption_share(&ciphertext.c3, transcript, &mut rng);

        DecryptionShare {
            participant_index: self.index,
            share0,
            share1,
            share2,
            share3,
        }
    }
}

impl DecryptionShare {
    #[allow(non_snake_case)]
    pub fn verify(
        &self,
        ctxt: &Ciphertext,
        pub_key_share: &PublicKeyShare,
        transcript: &mut merlin::Transcript,
    ) -> anyhow::Result<()> {
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

        self.share0.verify(&ctxt.c0, pub_key_share, transcript)?;
        self.share1.verify(&ctxt.c1, pub_key_share, transcript)?;
        self.share2.verify(&ctxt.c2, pub_key_share, transcript)?;
        self.share3.verify(&ctxt.c3, pub_key_share, transcript)?;

        Ok(())
    }
}
