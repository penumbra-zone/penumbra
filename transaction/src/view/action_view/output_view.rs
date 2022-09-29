use penumbra_crypto::{Note, PayloadKey};

pub struct OutputView {
    pub decrypted_note: Note,
    pub decrypted_memo_key: PayloadKey,
}
