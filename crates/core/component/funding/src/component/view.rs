use async_trait::async_trait;
use penumbra_sdk_txhash::TransactionId;

use crate::{component::state_key, params::FundingParameters};
use anyhow::Result;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};
use penumbra_sdk_sct::{component::clock::EpochRead, Nullifier};

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Gets the funding module chain parameters from the JMT.
    async fn get_staking_funding_params(&self) -> Result<FundingParameters> {
        self.get(state_key::staking_funding_parameters())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing FundingParameters"))
    }

    /// Gets the transaction id associated with the given nullifier from the JMT.
    async fn get_txid_from_nullifier(&self, nullifier: Nullifier) -> Option<TransactionId> {
        // Grab the ambient epoch index.
        let epoch_index = self
            .get_current_epoch()
            .await
            .expect("epoch is always set")
            .index;

        let nullifier_key = &state_key::lqt_nullifier_lookup_for_txid(epoch_index, &nullifier);

        let tx_id: Option<TransactionId> = self
            .nonverifiable_get(&nullifier_key.as_bytes())
            .await
            .expect("infallible");

        tx_id
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    /// Set the Funding parameters in the JMT.
    fn put_staking_funding_params(&mut self, params: FundingParameters) {
        self.put(state_key::staking_funding_parameters().into(), params)
    }

    // Sets the LQT nullifier in the JMT.
    fn put_lqt_nullifier(&mut self, epoch_index: u64, nullifier: Nullifier, tx_id: TransactionId) {
        let nullifier_key = state_key::lqt_nullifier_lookup_for_txid(epoch_index, &nullifier);

        self.put(nullifier_key, tx_id);
    }
}
impl<T: StateWrite + ?Sized> StateWriteExt for T {}
