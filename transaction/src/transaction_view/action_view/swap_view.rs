use penumbra_crypto::{dex::swap::SwapPlaintext, Note};

pub struct SwapView {
    pub swap_nft: Note,
    pub swap_plaintext: SwapPlaintext,
}
