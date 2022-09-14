use penumbra_crypto::{memo::MemoPlaintext, Note};

pub struct OutputView {
    pub decrypted_note: Note,
    pub memo: MemoPlaintext,
}
