use anyhow::ensure;
use cnidarium::{StateDelta, TempStorage};
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey};
use rand_core::OsRng;
use tendermint::PublicKey;

use crate::{
    component::{stake::address::validator_address, validator_handler::ValidatorDataRead},
    IdentityKey, StateWriteExt,
};

#[tokio::test]
/// Test that we do not delete rotated consensus keys from the [consensus key -> identity key] index.A
/// This is important to maintain because we want to be able to resolve byzantine evidence to a validator's
/// persistent identity even if they have rotated their consensus keys.
async fn test_persistent_identity_by_ck() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt::try_init();
    let storage = TempStorage::new().await?;
    let mut state = StateDelta::new(storage.latest_snapshot());

    let rng = OsRng;
    let vk = VerificationKey::from(SigningKey::<SpendAuth>::new(OsRng));
    let persistent_identity = IdentityKey(vk.into());

    let old_ck_raw = ed25519_consensus::SigningKey::new(rng)
        .verification_key()
        .to_bytes();
    let new_ck_raw = ed25519_consensus::SigningKey::new(rng)
        .verification_key()
        .to_bytes();

    let old_ck = PublicKey::from_raw_ed25519(&old_ck_raw).expect("valid vk");
    let new_ck = PublicKey::from_raw_ed25519(&new_ck_raw).expect("valid vk");
    anyhow::ensure!(
        old_ck.to_hex() != new_ck.to_hex(),
        "the keys must encode to different hex strings for the test to be useful"
    );

    let old_address = validator_address(&old_ck);
    let new_address = validator_address(&new_ck);

    state.register_consensus_key(&persistent_identity, &old_ck);

    let retrieved_ck = state
        .lookup_consensus_key_by_comet_address(&old_address)
        .await
        .expect("key is registered");

    ensure!(
        retrieved_ck == old_ck,
        "the retrieved consensus key must match the initial ck"
    );

    let retrieved_id = state
        .lookup_identity_key_by_consensus_key(&retrieved_ck)
        .await
        .expect("key is found");
    ensure!(
        retrieved_id == persistent_identity,
        "the retrieved identity must match its persistent identity"
    );

    state.register_consensus_key(&persistent_identity, &new_ck);
    // We want to do a basic check that we can reach for the updated values
    // but CRUCIALLY, we want to make sure that we can associate an identity to
    // the old consenus key.
    let retrieved_ck = state
        .lookup_consensus_key_by_comet_address(&new_address)
        .await
        .expect("key is registered");
    ensure!(
        retrieved_ck == new_ck,
        "we must be able to find the updated ck"
    );

    let retrieved_id = state
        .lookup_identity_key_by_consensus_key(&retrieved_ck)
        .await
        .expect("key is found");
    ensure!(
        retrieved_id == persistent_identity,
        "the retrieved id must match the persistent identity, even after an update"
    );

    // CRUCIAL PART: can we find the persistent identity from a rotated comet address/consensus key?
    let culprit_ck = state
        .lookup_consensus_key_by_comet_address(&old_address)
        .await
        .expect("key must be found!");
    ensure!(
        culprit_ck == old_ck,
        "the old address must be associated with the old ck"
    );

    let culprit_id = state
        .lookup_identity_key_by_consensus_key(&culprit_ck)
        .await
        .expect("consensus key -> identity index is persistent across validator updates");
    ensure!(
        culprit_id == persistent_identity,
        "the retrieved identity must match the persistent identity that we setup"
    );

    Ok(())
}
