use anyhow::{anyhow, Result};
use penumbra_tct as tct;

use crate::{keys, note, Nullifier};

/// Check the integrity of the nullifier.
pub(crate) fn nullifier_integrity(
    public_nullifier: Nullifier,
    nk: keys::NullifierKey,
    position: tct::Position,
    note_commitment: note::Commitment,
) -> Result<()> {
    if public_nullifier != nk.derive_nullifier(position, &note_commitment) {
        return Err(anyhow!("bad nullifier"));
    }
    Ok(())
}
