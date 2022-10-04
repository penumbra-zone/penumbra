use penumbra_crypto::{Note, PayloadKey};

use crate::action::Output;
#[allow(clippy::large_enum_variant)]
pub enum OutputView {
    Visible {
        output: Output,
        decrypted_note: Note,
        decrypted_memo_key: PayloadKey,
    },
    Opaque {
        output: Output,
    },
}
