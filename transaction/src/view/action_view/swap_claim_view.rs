use penumbra_crypto::Note;

use crate::action::SwapClaim;
#[allow(clippy::large_enum_variant)]
pub enum SwapClaimView {
    Visible {
        swap_claim: SwapClaim,
        decrypted_note_1: Note,
        decrypted_note_2: Note,
    },
    Opaque {
        swap_claim: SwapClaim,
    },
}
