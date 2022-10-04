use penumbra_crypto::Note;

use crate::action::Spend;
#[allow(clippy::large_enum_variant)]
pub enum SpendView {
    Visible { spend: Spend, note: Note },
    Opaque { spend: Spend },
}
