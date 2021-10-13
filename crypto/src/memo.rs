use crate::{
    addresses::PaymentAddress,
    constants::{MEMO_CIPHERTEXT_LEN_BYTES, MEMO_LEN_BYTES},
};

// The memo is stored separately from the `Note`.
pub struct MemoPlaintext([u8; MEMO_LEN_BYTES]);

impl Default for MemoPlaintext {
    fn default() -> MemoPlaintext {
        MemoPlaintext([0u8; MEMO_LEN_BYTES])
    }
}

impl MemoPlaintext {
    pub fn encrypt(&self, key: &PaymentAddress) -> MemoCiphertext {
        todo!()
    }
}

pub struct MemoCiphertext([u8; MEMO_CIPHERTEXT_LEN_BYTES]);
