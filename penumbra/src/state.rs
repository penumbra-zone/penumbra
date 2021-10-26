//! In-memory storage of state for MVP1 of the Penumbra node software.
use std::collections::HashSet;

use penumbra_crypto::{merkle, merkle::TreeExt, note, Action, Nullifier, Transaction};

pub const MAX_MERKLE_CHECKPOINTS: usize = 100;

pub struct FullNodeState {
    node_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,
    nullifier_set: HashSet<Nullifier>,
}

impl FullNodeState {
    pub fn new() -> Self {
        Self {
            node_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS),
            // TODO: Store cached merkle root to prevent recomputing it - currently
            // this is happening for each spend (since we pass in the merkle_root when
            // verifying the spend proof).
            nullifier_set: HashSet::new(),
        }
    }

    /// Verifies a transaction and if it verifies, updates the node state.
    pub fn verify_transaction(&mut self, transaction: Transaction) -> bool {
        // 1. Check binding signature.
        if !transaction.verify_binding_sig() {
            return false;
        }

        // 2. Check all spend auth signatures using provided spend auth keys
        // and check all proofs verify. If any action does not verify, the entire
        // transaction has failed.
        for action in transaction.transaction_body().actions {
            match action {
                Action::Output(inner) => {
                    if !inner.body.proof.verify(
                        inner.body.value_commitment,
                        inner.body.note_commitment,
                        inner.body.ephemeral_key,
                    ) {
                        return false;
                    }
                }
                Action::Spend(inner) => {
                    if !inner.verify_auth_sig() {
                        return false;
                    }

                    if !inner.body.proof.verify(
                        self.node_commitment_tree.root2(),
                        inner.body.value_commitment,
                        inner.body.nullifier,
                        inner.body.rk,
                    ) {
                        return false;
                    }
                }
            }
        }

        // 3. Update note state.
        todo!();

        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_ff::Zero;
    use rand_core::OsRng;

    use penumbra_crypto::{keys::SpendKey, memo::MemoPlaintext, Fq, Note, Value};

    #[test]
    fn test_transaction_verification() {
        let mut state = FullNodeState::new();

        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ovk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (dest, _dtk_d) = ivk_recipient.payment_address(0u64.into());

        let merkle_root = merkle::Root(Fq::zero());
        let mut merkle_siblings = Vec::new();
        for _i in 0..merkle::DEPTH {
            merkle_siblings.push(note::Commitment(Fq::zero()))
        }
        let dummy_merkle_path: merkle::Path = (merkle::DEPTH, merkle_siblings);

        let value_to_send = Value {
            amount: 10,
            asset_id: b"pen".as_ref().into(),
        };
        let dummy_note = Note::new(
            *dest.diversifier(),
            dest.transmission_key(),
            value_to_send,
            Fq::zero(),
        )
        .expect("transmission key is valid");

        let transaction = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .set_chain_id("Pen".to_string())
            .add_output(
                &mut rng,
                &dest,
                value_to_send,
                MemoPlaintext::default(),
                ovk_sender,
            )
            .add_spend(&mut rng, sk_sender, dummy_merkle_path, dummy_note, 0.into())
            .finalize(&mut rng);

        assert!(transaction.is_ok());

        // Now we verify this transaction.
        //state.verify_transaction(transaction.unwrap());
    }
}
