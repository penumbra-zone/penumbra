use anyhow::{anyhow, Result};
use decaf377_fmd as fmd;
use penumbra_tct as tct;

use crate::{balance, keys, note, Fq, Fr, Nullifier, Value};

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

/// Check the integrity of the note commitment.
pub(crate) fn note_commitment_integrity(
    note_blinding: Fq,
    note_value: Value,
    note_diversified_generator: decaf377::Element,
    note_s_component_transmission_key: Fq,
    note_clue_key: fmd::ClueKey,
    note_commitment: note::Commitment,
) -> Result<()> {
    let note_commitment_test = note::commitment(
        note_blinding,
        note_value,
        note_diversified_generator,
        note_s_component_transmission_key,
        &note_clue_key,
    );

    if note_commitment != note_commitment_test {
        return Err(anyhow!("note commitment mismatch"));
    }
    Ok(())
}

/// Check the integrity of the value commitment.
pub(crate) fn value_commitment_integrity(
    balance_commitment: balance::Commitment,
    value_blinding: Fr,
    value: Value,
) -> Result<()> {
    if balance_commitment != value.commit(value_blinding) {
        return Err(anyhow!("value commitment mismatch"));
    }

    Ok(())
}
