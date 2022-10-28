use anyhow::{anyhow, Result};
use decaf377_rdsa::{SpendAuth, VerificationKey};
use penumbra_tct as tct;

use crate::{
    asset, balance, dex, ka, keys, note, transaction::Fee, Address, Fr, Note, Nullifier, Balance,
};

/// Check the integrity of the nullifier.
pub(crate) fn nullifier_integrity(
    public_nullifier: Nullifier,
    nk: keys::NullifierKey,
    position: tct::Position,
    note_commitment: note::Commitment,
) -> Result<()> {
    if public_nullifier != nk.derive_nullifier(position, &note_commitment) {
        Err(anyhow!("bad nullifier"))
    } else {
        Ok(())
    }
}

/// Check the integrity of the note commitment.
pub(crate) fn note_commitment_integrity(
    note: Note,
    note_commitment: note::Commitment,
) -> Result<()> {
    let note_commitment_test = note::commitment(
        note.note_blinding(),
        note.value(),
        note.diversified_generator(),
        note.transmission_key_s(),
        note.clue_key(),
    );

    if note_commitment != note_commitment_test {
        Err(anyhow!("note commitment mismatch"))
    } else {
        Ok(())
    }
}

/// Check the integrity of the balance (previously value) commitment.
pub(crate) fn balance_commitment_integrity(
    balance_commitment: balance::Commitment,
    value_blinding: Fr,
    balance: Balance,
) -> Result<()> {
    if balance_commitment != balance.commit(value_blinding) {
        Err(anyhow!("balance commitment mismatch"))
    } else {
        Ok(())
    }
}

/// Check the integrity of an ephemeral public key.
pub(crate) fn ephemeral_public_key_integrity(
    public_key: ka::Public,
    secret_key: ka::Secret,
    diversified_generator: decaf377::Element,
) -> Result<()> {
    if secret_key.diversified_public(&diversified_generator) != public_key {
        Err(anyhow!("ephemeral public key mismatch"))
    } else {
        Ok(())
    }
}

/// Check the integrity of a diversified address given a `Note`.
pub(crate) fn diversified_address_integrity(
    ak: VerificationKey<SpendAuth>,
    nk: keys::NullifierKey,
    note: Note,
) -> Result<()> {
    let transmission_key = note.transmission_key();
    let diversified_generator = note.diversified_generator();
    let fvk = keys::FullViewingKey::from_components(ak, nk);
    let ivk = fvk.incoming();
    if *transmission_key != ivk.diversified_public(&diversified_generator) {
        Err(anyhow!("invalid diversified address"))
    } else {
        Ok(())
    }
}

/// Check the integrity of the asset ID of a swap NFT.
pub(crate) fn swap_nft_asset_id_integrity(
    asset_id: asset::Id,
    trading_pair: dex::TradingPair,
    delta_1_i: u64,
    delta_2_i: u64,
    fee: Fee,
    claim_address: Address,
) -> Result<()> {
    let expected_plaintext = dex::swap::SwapPlaintext::from_parts(
        trading_pair,
        delta_1_i.into(),
        delta_2_i.into(),
        fee,
        // This should ensure that the claim address matches the address
        // used to construct the Swap NFT.
        claim_address,
    )
    .map_err(|_| anyhow!("error generating expected swap plaintext"))?;
    let expected_asset_id = expected_plaintext.asset_id();
    if expected_asset_id != asset_id {
        Err(anyhow!("improper swap NFT asset id"))
    } else {
        Ok(())
    }
}

/// Check diversified basepoint is not identity.
///
/// The use of decaf means that we do not need to check that the
/// diversified basepoint is of small order, we instead check it is not identity.
pub(crate) fn diversified_basepoint_not_identity(point: decaf377::Element) -> Result<()> {
    if point.is_identity() {
        Err(anyhow!("unexpected identity"))
    } else {
        Ok(())
    }
}

/// Check randomized verification key is generated from the spend verification key `ak`.
pub(crate) fn rk_integrity(
    spend_auth_randomizer: Fr,
    rk: VerificationKey<SpendAuth>,
    ak: VerificationKey<SpendAuth>,
) -> Result<()> {
    let rk_bytes: [u8; 32] = rk.into();
    let rk_test = ak.randomize(&spend_auth_randomizer);
    let rk_test_bytes: [u8; 32] = rk_test.into();
    if rk_bytes != rk_test_bytes {
        Err(anyhow!("invalid spend auth randomizer"))
    } else {
        Ok(())
    }
}
