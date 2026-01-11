use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_compliance::registry::ComplianceRegistryRead;
use penumbra_sdk_proof_params::SPEND_PROOF_VERIFICATION_KEY;
use penumbra_sdk_proto::{DomainType, StateWriteProto as _};
use penumbra_sdk_sct::component::{
    clock::EpochRead,
    source::SourceContext,
    tree::{SctManager, VerificationExt},
};
use penumbra_sdk_txhash::TransactionContext;

use crate::{event, Spend, SpendProofPublic};

/// Maximum allowed time difference (in seconds) between block timestamp and target_timestamp.
/// Transactions outside this window will be rejected.
const MAX_TIMESTAMP_DELTA_SECONDS: u64 = 3600; // 1 hour

#[async_trait]
impl ActionHandler for Spend {
    type CheckStatelessContext = TransactionContext;

    async fn check_stateless(&self, context: TransactionContext) -> Result<()> {
        let spend = self;

        // 1. Check spend auth signature using provided spend auth key.
        spend
            .body
            .rk
            .verify(context.effect_hash.as_ref(), &spend.auth_sig)
            .context("spend auth signature failed to verify")?;

        // 2. Check that the proof verifies.
        // Use anchors from the action body (set during proof generation).
        // The stateful check will validate these anchors against chain state.
        let asset_anchor = spend.body.asset_anchor;
        let compliance_anchor = spend.body.compliance_anchor;

        // Extract compliance ciphertext using unified method
        use penumbra_sdk_compliance::structs::ComplianceCiphertext;
        let ct = ComplianceCiphertext::from_bytes(&spend.body.compliance_ciphertext)
            .context("failed to deserialize compliance ciphertext")?;
        let (compliance_epk, compliance_ciphertext) = ct.to_circuit_public_inputs();

        let public = SpendProofPublic {
            anchor: context.anchor,
            balance_commitment: spend.body.balance_commitment,
            nullifier: spend.body.nullifier,
            rk: spend.body.rk,
            asset_anchor,
            compliance_anchor,
            compliance_epk,
            compliance_ciphertext,
            target_timestamp: spend.body.target_timestamp,
            sender_leaf_hash: spend.body.sender_leaf_hash,
            counterparty_leaf_hash: spend.body.counterparty_leaf_hash,
        };

        spend
            .proof
            .verify(&SPEND_PROOF_VERIFICATION_KEY, public)
            .context("a spend proof did not verify")?;

        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // 1. Validate target_timestamp is within acceptable window
        let block_time = state.get_current_block_timestamp().await?;
        let block_timestamp = block_time.unix_timestamp() as u64;
        let target_timestamp = self.body.target_timestamp;

        let delta = if block_timestamp >= target_timestamp {
            block_timestamp - target_timestamp
        } else {
            target_timestamp - block_timestamp
        };

        if delta > MAX_TIMESTAMP_DELTA_SECONDS {
            return Err(anyhow!(
                "target_timestamp {} is outside acceptable window (block time: {}, delta: {} seconds, max: {} seconds)",
                target_timestamp,
                block_timestamp,
                delta,
                MAX_TIMESTAMP_DELTA_SECONDS
            ));
        }

        // 2. Enforce Compliance: Validate anchors match chain state.
        // The proof was already verified in check_stateless using the anchors from body.
        // Here we validate that those anchors are valid (i.e., they match or are ancestors of current state).
        let chain_compliance_anchor = state.get_user_tree_root().await?;
        let chain_asset_anchor = state.get_asset_tree_root().await?;

        // For now, require exact match. In the future, we could allow historical anchors.
        if self.body.compliance_anchor != chain_compliance_anchor {
            return Err(anyhow!(
                "spend compliance_anchor {:?} does not match chain state {:?}",
                self.body.compliance_anchor,
                chain_compliance_anchor
            ));
        }
        if self.body.asset_anchor != chain_asset_anchor {
            return Err(anyhow!(
                "spend asset_anchor {:?} does not match chain state {:?}",
                self.body.asset_anchor,
                chain_asset_anchor
            ));
        }

        // TODO: Transaction-level validation of blinded leaf hashes
        // After all spend and output proofs are verified individually, the transaction validator
        // must verify the cryptographic binding between spend and output circuits:
        //   - For each spend/output pair in the transaction:
        //     * spend.counterparty_leaf_hash MUST equal output.receiver_leaf_hash
        //     * output.counterparty_leaf_hash MUST equal spend.sender_leaf_hash
        // This ensures that the same tx_blinding_nonce was used in both circuits and that
        // the counterparty relationship is cryptographically bound without leaking which
        // compliance leaves are transacting (due to the blinding).

        // 3. Check that the `Nullifier` has not been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await?;

        let source = state.get_current_source().expect("source should be set");

        state.nullify(self.body.nullifier, source.into()).await;

        // Also record an ABCI event for transaction indexing.
        state.record_proto(
            event::EventSpend {
                nullifier: self.body.nullifier,
            }
            .to_proto(),
        );

        Ok(())
    }
}
