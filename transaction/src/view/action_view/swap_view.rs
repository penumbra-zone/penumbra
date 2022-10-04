use penumbra_crypto::{dex::swap::SwapPlaintext, Note};

use crate::action::Swap;
#[allow(clippy::large_enum_variant)]
pub enum SwapView {
    Visible {
        swap: Swap,
        swap_nft: Note,
        swap_plaintext: SwapPlaintext,
    },
    Opaque {
        swap: Swap,
    },
}
