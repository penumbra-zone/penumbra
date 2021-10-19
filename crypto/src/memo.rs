use crate::addresses::PaymentAddress;

pub const MEMO_CIPHERTEXT_LEN_BYTES: usize = 528;

// This is the `MEMO_CIPHERTEXT_LEN_BYTES` - MAC size (16 bytes).
pub const MEMO_LEN_BYTES: usize = 512;

// The memo is stored separately from the `Note`.
pub struct MemoPlaintext(pub [u8; MEMO_LEN_BYTES]);

impl Default for MemoPlaintext {
    fn default() -> MemoPlaintext {
        MemoPlaintext([0u8; MEMO_LEN_BYTES])
    }
}

impl MemoPlaintext {
    pub fn encrypt(&self, _key: &PaymentAddress) -> MemoCiphertext {
        todo!()
    }
}

pub struct MemoCiphertext(pub [u8; MEMO_CIPHERTEXT_LEN_BYTES]);
