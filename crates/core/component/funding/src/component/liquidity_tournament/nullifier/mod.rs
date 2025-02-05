use async_trait::async_trait;
use penumbra_sdk_txhash::TransactionId;

use crate::component::state_key;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_sct::{component::clock::EpochRead, Nullifier};

#[async_trait]
pub trait NullifierRead: StateRead {
    /// Returns the `TransactionId` if the nullifier has been spent in the current epoch;
    /// Otherwise, returns None.
    async fn get_lqt_spent_nullifier(&self, nullifier: Nullifier) -> Option<TransactionId> {
        // Grab the ambient epoch index.
        let epoch_index = self
            .get_current_epoch()
            .await
            .expect("epoch is always set")
            .index;

        let nullifier_key = &state_key::lqt::v1::nullifier::key(epoch_index, &nullifier);

        let tx_id: Option<TransactionId> = self
            .nonverifiable_get(&nullifier_key.as_bytes())
            .await
            .expect(&format!(
                "failed to retrieve key {} from non-verifiable storage",
                nullifier_key,
            ));

        tx_id
    }

    /// Returns the `TransactionId` if the nullifier has been spent in a specific epoch;
    /// Otherwise, returns None.
    async fn get_lqt_spent_nullifier_by_epoch(
        &self,
        nullifier: Nullifier,
        epoch_index: u64,
    ) -> Option<TransactionId> {
        let nullifier_key = &state_key::lqt::v1::nullifier::key(epoch_index, &nullifier);

        let tx_id: Option<TransactionId> = self
            .nonverifiable_get(&nullifier_key.as_bytes())
            .await
            .expect(&format!(
                "failed to retrieve key {} from non-verifiable storage",
                nullifier_key,
            ));

        tx_id
    }
}

impl<T: StateRead + ?Sized> NullifierRead for T {}

#[async_trait]
pub trait NullifierWrite: StateWrite {
    /// Sets the LQT nullifier in the NV storage.
    fn put_lqt_spent_nullifier(
        &mut self,
        epoch_index: u64,
        nullifier: Nullifier,
        tx_id: TransactionId,
    ) {
        let nullifier_key = state_key::lqt::v1::nullifier::key(epoch_index, &nullifier);

        self.nonverifiable_put(nullifier_key.into(), tx_id);
    }
}

impl<T: StateWrite + ?Sized> NullifierWrite for T {}
