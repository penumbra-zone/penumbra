use async_trait::async_trait;
use penumbra_sdk_txhash::TransactionId;

use crate::component::state_key;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_sct::{component::clock::EpochRead, Nullifier};

#[allow(dead_code)]
#[async_trait]
pub trait NullifierRead: StateRead {
    /// Gets the transaction id associated with the given nullifier from the JMT.
    async fn get_txid_from_nullifier(&self, nullifier: Nullifier) -> Option<TransactionId> {
        // Grab the ambient epoch index.
        let epoch_index = self
            .get_current_epoch()
            .await
            .expect("epoch is always set")
            .index;

        let nullifier_key =
            &state_key::lqt::v1::nullifier::lqt_nullifier_lookup_for_txid(epoch_index, &nullifier);

        let tx_id: Option<TransactionId> = self
            .nonverifiable_get(&nullifier_key.as_bytes())
            .await
            .expect("infallible");

        tx_id
    }
}

impl<T: StateRead + ?Sized> NullifierRead for T {}

#[allow(dead_code)]
#[async_trait]
pub trait NullifierWrite: StateWrite {
    /// Sets the LQT nullifier in the JMT.
    fn put_lqt_nullifier(&mut self, epoch_index: u64, nullifier: Nullifier, tx_id: TransactionId) {
        let nullifier_key =
            state_key::lqt::v1::nullifier::lqt_nullifier_lookup_for_txid(epoch_index, &nullifier);

        self.put(nullifier_key, tx_id);
    }
}

impl<T: StateWrite + ?Sized> NullifierWrite for T {}
