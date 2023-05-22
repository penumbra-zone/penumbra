use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
};

use anyhow::anyhow;
use rand_core::OsRng;

use crate::{
    balance, ka,
    keys::OutgoingViewingKey,
    note,
    symmetric::{OvkWrappedKey, PayloadKey, PayloadKind, WrappedMemoKey},
    Address, Note,
};
use penumbra_proto::core::transaction::v1alpha1 as pbt;
pub const MEMO_CIPHERTEXT_LEN_BYTES: usize = 528;

// This is the `MEMO_CIPHERTEXT_LEN_BYTES` - MAC size (16 bytes).
pub const MEMO_LEN_BYTES: usize = 512;

#[derive(Clone, Debug)]
pub struct MemoCiphertext(pub [u8; MEMO_CIPHERTEXT_LEN_BYTES]);

#[derive(Clone, Debug, PartialEq)]
pub struct MemoPlaintext {
    pub sender: Address,
    pub text: String,
}

impl From<&MemoPlaintext> for Vec<u8> {
    fn from(plaintext: &MemoPlaintext) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&plaintext.sender.to_vec());
        bytes.extend_from_slice(plaintext.text.as_bytes());
        bytes
    }
}

impl TryFrom<Vec<u8>> for MemoPlaintext {
    type Error = anyhow::Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() < 80 {
            return Err(anyhow!("malformed memo plaintext: missing sender address"));
        }
        let sender_address_bytes = &bytes[..80];
        let sender_address: Address = sender_address_bytes.try_into()?;
        let text = String::from_utf8_lossy(&bytes[80..])
            .trim_end_matches(0u8 as char)
            .to_string();

        Ok(MemoPlaintext {
            sender: sender_address,
            text,
        })
    }
}

impl Default for MemoPlaintext {
    fn default() -> Self {
        let mut rng = OsRng;
        MemoPlaintext {
            sender: Address::dummy(&mut rng),
            text: String::new(),
        }
    }
}

impl MemoPlaintext {
    pub fn to_vec(&self) -> Vec<u8> {
        self.into()
    }
}

impl MemoCiphertext {
    /// Encrypt a memo, returning its ciphertext.
    pub fn encrypt(
        memo_key: PayloadKey,
        memo: &MemoPlaintext,
    ) -> Result<MemoCiphertext, anyhow::Error> {
        let memo_bytes: Vec<u8> = memo.into();
        let memo_len = memo_bytes.len();
        if memo_len > MEMO_LEN_BYTES {
            return Err(anyhow::anyhow!(
                "provided memo plaintext of length {memo_len} exceeds maximum memo length of {MEMO_LEN_BYTES}"
            ));
        }
        let mut m = [0u8; MEMO_LEN_BYTES];
        m[..memo_len].copy_from_slice(&memo_bytes);

        let encryption_result = memo_key.encrypt(m.to_vec(), PayloadKind::Memo);
        let ciphertext: [u8; MEMO_CIPHERTEXT_LEN_BYTES] = encryption_result
            .try_into()
            .expect("memo encryption result fits in ciphertext len");

        Ok(MemoCiphertext(ciphertext))
    }

    /// Decrypt a [`MemoCiphertext`] to generate a plaintext [`MemoPlaintext`].
    pub fn decrypt(
        memo_key: &PayloadKey,
        ciphertext: MemoCiphertext,
    ) -> Result<MemoPlaintext, anyhow::Error> {
        let plaintext_bytes = MemoCiphertext::decrypt_bytes(memo_key, ciphertext)?;

        let sender_address_bytes = &plaintext_bytes[..80];
        let sender_address: Address = sender_address_bytes.try_into()?;
        let text = String::from_utf8_lossy(&plaintext_bytes[80..])
            .trim_end_matches(0u8 as char)
            .to_string();

        Ok(MemoPlaintext {
            sender: sender_address,
            text,
        })
    }

    /// Decrypt a [`MemoCiphertext`] to generate a fixed-length slice of bytes.
    pub fn decrypt_bytes(
        memo_key: &PayloadKey,
        ciphertext: MemoCiphertext,
    ) -> Result<[u8; MEMO_LEN_BYTES], anyhow::Error> {
        let decryption_result = memo_key
            .decrypt(ciphertext.0.to_vec(), PayloadKind::Memo)
            .map_err(|_| anyhow!("decryption error"))?;
        let plaintext_bytes: [u8; MEMO_LEN_BYTES] = decryption_result.try_into().map_err(|_| {
            anyhow!("post-decryption, could not fit plaintext into memo size {MEMO_LEN_BYTES}")
        })?;
        Ok(plaintext_bytes)
    }

    /// Decrypt a [`MemoCiphertext`] using the wrapped OVK to generate a plaintext [`Memo`].
    pub fn decrypt_outgoing(
        wrapped_memo_key: &WrappedMemoKey,
        wrapped_ovk: OvkWrappedKey,
        cm: note::Commitment,
        cv: balance::Commitment,
        ovk: &OutgoingViewingKey,
        epk: &ka::Public,
        ciphertext: MemoCiphertext,
    ) -> Result<MemoPlaintext, anyhow::Error> {
        let shared_secret = Note::decrypt_key(wrapped_ovk, cm, cv, ovk, epk)
            .map_err(|_| anyhow!("key decryption error"))?;

        let action_key = PayloadKey::derive(&shared_secret, epk);
        let memo_key = wrapped_memo_key
            .decrypt_outgoing(&action_key)
            .map_err(|_| anyhow!("could not decrypt wrapped memo key"))?;

        let plaintext = memo_key
            .decrypt(ciphertext.0.to_vec(), PayloadKind::Memo)
            .map_err(|_| anyhow!("decryption error"))?;

        let plaintext_bytes: [u8; MEMO_LEN_BYTES] = plaintext.try_into().map_err(|_| {
            anyhow!("post-decryption, could not fit plaintext into memo size {MEMO_LEN_BYTES}")
        })?;

        let sender_address_bytes = &plaintext_bytes[..80];
        let sender_address: Address = sender_address_bytes.try_into()?;
        let text = String::from_utf8_lossy(&plaintext_bytes[80..])
            .trim_end_matches(0u8 as char)
            .to_string();

        Ok(MemoPlaintext {
            sender: sender_address,
            text,
        })
    }
}

impl TryFrom<&[u8]> for MemoCiphertext {
    type Error = anyhow::Error;

    fn try_from(input: &[u8]) -> Result<MemoCiphertext, Self::Error> {
        if input.len() > MEMO_CIPHERTEXT_LEN_BYTES {
            return Err(anyhow::anyhow!(
                "provided memo ciphertext exceeds maximum memo size"
            ));
        }
        let mut mc = [0u8; MEMO_CIPHERTEXT_LEN_BYTES];
        mc[..input.len()].copy_from_slice(input);

        Ok(MemoCiphertext(mc))
    }
}

impl From<MemoPlaintext> for pbt::MemoPlaintext {
    fn from(plaintext: MemoPlaintext) -> pbt::MemoPlaintext {
        pbt::MemoPlaintext {
            sender: Some(plaintext.sender.into()),
            text: plaintext.text,
        }
    }
}

impl TryFrom<pbt::MemoCiphertext> for MemoCiphertext {
    type Error = anyhow::Error;

    fn try_from(msg: pbt::MemoCiphertext) -> Result<Self, Self::Error> {
        MemoCiphertext::try_from(msg.inner.to_vec().as_slice())
    }
}

impl From<MemoCiphertext> for pbt::MemoCiphertext {
    fn from(ciphertext: MemoCiphertext) -> pbt::MemoCiphertext {
        pbt::MemoCiphertext {
            inner: ciphertext.0.to_vec().into(),
        }
    }
}

impl TryFrom<pbt::MemoPlaintext> for MemoPlaintext {
    type Error = anyhow::Error;

    fn try_from(msg: pbt::MemoPlaintext) -> Result<Self, Self::Error> {
        let sender = msg
            .sender
            .ok_or_else(|| anyhow::anyhow!("message missing sender address"))?
            .try_into()?;
        Ok(Self {
            sender,
            text: msg.text,
        })
    }
}

#[cfg(test)]
mod tests {
    use ark_ff::UniformRand;
    use rand_core::OsRng;

    use super::*;
    use crate::{
        asset,
        keys::{SeedPhrase, SpendKey},
        Value,
    };
    use decaf377::Fr;

    use proptest::prelude::*;

    #[test]
    fn test_memo_encryption_and_decryption() {
        let mut rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let esk = ka::Secret::new(&mut rng);

        // On the sender side, we have to encrypt the memo to put into the transaction-level,
        // and also the memo key to put on the action-level (output).
        let memo = MemoPlaintext {
            sender: dest,
            text: String::from("Hi"),
        };
        let memo_key = PayloadKey::random_key(&mut OsRng);
        let ciphertext =
            MemoCiphertext::encrypt(memo_key.clone(), &memo).expect("can encrypt memo");
        let wrapped_memo_key = WrappedMemoKey::encrypt(
            &memo_key,
            esk.clone(),
            dest.transmission_key(),
            dest.diversified_generator(),
        );

        // On the recipient side, we have to decrypt the wrapped memo key, and then the memo.
        let epk = esk.diversified_public(dest.diversified_generator());
        let decrypted_memo_key = wrapped_memo_key
            .decrypt(epk, ivk)
            .expect("can decrypt memo key");
        let plaintext =
            MemoCiphertext::decrypt(&decrypted_memo_key, ciphertext).expect("can decrypt memo");

        assert_eq!(memo_key, decrypted_memo_key);
        assert_eq!(plaintext, memo);
    }

    #[test]
    fn test_memo_encryption_and_sender_decryption() {
        let mut rng = OsRng;

        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let ovk = fvk.outgoing();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let value = Value {
            amount: 10u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };
        let note = Note::generate(&mut rng, &dest, value);

        // On the sender side, we have to encrypt the memo to put into the transaction-level,
        // and also the memo key to put on the action-level (output).
        let memo = MemoPlaintext {
            sender: dest,
            text: String::from("Hello, friend"),
        };
        let memo_key = PayloadKey::random_key(&mut OsRng);
        let ciphertext =
            MemoCiphertext::encrypt(memo_key.clone(), &memo).expect("can encrypt memo");
        let esk = note.ephemeral_secret_key();
        let wrapped_memo_key = WrappedMemoKey::encrypt(
            &memo_key,
            esk.clone(),
            dest.transmission_key(),
            dest.diversified_generator(),
        );

        let value_blinding = Fr::rand(&mut rng);
        let cv = note.value().commit(value_blinding);
        let wrapped_ovk = note.encrypt_key(ovk, cv);

        // Later, still on the sender side, we decrypt the memo by using the decrypt_outgoing method.
        let epk = esk.diversified_public(dest.diversified_generator());
        let plaintext = MemoCiphertext::decrypt_outgoing(
            &wrapped_memo_key,
            wrapped_ovk,
            note.commit(),
            cv,
            ovk,
            &epk,
            ciphertext,
        )
        .expect("can decrypt memo");

        assert_eq!(plaintext, memo);
    }

    proptest! {
        // We generate random strings, up to 10k chars long.
        // Since UTF-8 represents each char using 1 to 4 bytes,
        // we need to test strings up to (MEMO_LEN_BYTES * 4 = 2048)
        // chars in length. That's the intended upper bound of what
        // the memo parsing will handle, but for the sake of tests,
        // let's raise it 2048 -> 10,000. Doing so only adds a fraction
        // of a second to the length of the test run.
        #[test]
        fn test_memo_size_limit(s in "\\PC{0,10000}") {
            let mut rng = OsRng;
            let memo_key = PayloadKey::random_key(&mut rng);
            let memo_address = Address::dummy(&mut rng);
            let memo_text = s;
            let memo = MemoPlaintext {
                sender: memo_address,
                text: memo_text,
            };
            let ciphertext_result = MemoCiphertext::encrypt(memo_key.clone(), &memo);
            if memo.to_vec().len() > MEMO_LEN_BYTES {
                assert!(ciphertext_result.is_err());
            } else {
                assert!(ciphertext_result.is_ok());
                let plaintext = MemoCiphertext::decrypt(&memo_key, ciphertext_result.unwrap()).unwrap();
                assert_eq!(plaintext, memo);
            }
        }
    }
}
