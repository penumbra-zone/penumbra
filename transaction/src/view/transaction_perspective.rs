use std::collections::BTreeMap;

use penumbra_crypto::{
    memo::{MemoCiphertext, MemoPlaintext},
    note, Note, Nullifier, PayloadKey,
};

/// This represents the data to understand an individual transaction without
/// disclosing viewing keys.
pub struct TransactionPerspective {
    /// List of per-action payload keys. These can be used to decrypt
    /// the notes, swaps, and memo keys in the transaction.
    ///
    /// One-to-one correspondence between:
    /// * Output and note,
    /// * Swap and note (NFT),
    ///
    /// There is not a one-to-one correspondence between SwapClaim and notes,
    /// i.e. there are two notes per SwapClaim.
    ///
    /// For outputs, we can use the PayloadKey associated with that output
    /// to decrypt the wrapped_memo_key, which will be used to decrypt the
    /// memo in the transaction. This needs to be done only once, because
    /// there is one memo shared between all outputs.
    pub payload_keys: BTreeMap<note::Commitment, PayloadKey>,
    /// Mapping of nullifiers spent in this transaction to notes.
    pub spend_nullifiers: BTreeMap<Nullifier, Note>,

    pub memo_cipher_text: Option<MemoCiphertext>,

    pub memo_plaintext: Option<MemoPlaintext>,
}

impl TransactionPerspective {}
