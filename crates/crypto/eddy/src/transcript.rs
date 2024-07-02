use crate::{limb, PublicKeyShare};

pub trait TranscriptProtocol {
    fn begin_decryption(&mut self);
    fn begin_limb_decryption(&mut self);
    fn append_public_key_share(&mut self, share: &PublicKeyShare);
    fn append_limb_ciphertext(&mut self, ciphertext: &limb::Ciphertext);
    fn append_decryption_share_point(&mut self, point: &decaf377::Element);
    fn append_blinding_commitment(&mut self, label: &'static [u8], point: &decaf377::Element);

    fn challenge_scalar(&mut self, label: &'static [u8]) -> decaf377::Fr;
}

impl TranscriptProtocol for merlin::Transcript {
    fn begin_decryption(&mut self) {
        self.append_message(b"dom-sep", b"eddy-decaf377-decrypt");
    }
    fn begin_limb_decryption(&mut self) {
        self.append_message(b"dom-sep", b"begin-limb");
    }
    fn append_public_key_share(&mut self, share: &PublicKeyShare) {
        self.append_message(b"dom-sep", b"public-key-share");
        self.append_message(b"index", &share.participant_index.to_le_bytes());
        self.append_message(
            b"public-key-share",
            &share.pub_key_share.vartime_compress().0,
        );
    }
    fn append_limb_ciphertext(&mut self, ciphertext: &limb::Ciphertext) {
        self.append_message(b"dom-sep", b"limb-ciphertext");
        self.append_message(b"c1", &ciphertext.c1.vartime_compress().0);
        self.append_message(b"c2", &ciphertext.c2.vartime_compress().0);
    }
    fn append_decryption_share_point(&mut self, point: &decaf377::Element) {
        self.append_message(b"decryption-share-point", &point.vartime_compress().0);
    }
    fn append_blinding_commitment(&mut self, label: &'static [u8], point: &decaf377::Element) {
        self.append_message(b"dom-sep", label);
        self.append_message(b"blinding-commitment", &point.vartime_compress().0);
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> decaf377::Fr {
        let mut bytes = [0; 64];
        self.challenge_bytes(label, &mut bytes);
        decaf377::Fr::from_le_bytes_mod_order(&bytes)
    }
}
