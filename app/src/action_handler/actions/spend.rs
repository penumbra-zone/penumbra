use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_proof_params::SPEND_PROOF_VERIFICATION_KEY;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::Spend, Transaction};

use crate::shielded_pool::StateReadExt;
use crate::{action_handler::ActionHandler, shielded_pool::NoteManager};

#[async_trait]
impl ActionHandler for Spend {
    async fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        let spend = self;
        let effect_hash = context.transaction_body().effect_hash();
        let anchor = context.anchor;

        // 2. Check spend auth signature using provided spend auth key.
        spend
            .body
            .rk
            .verify(effect_hash.as_ref(), &spend.auth_sig)
            .context("spend auth signature failed to verify")?;

        // 3. Check that the proof verifies.
        spend
            .proof
            .verify(
                &SPEND_PROOF_VERIFICATION_KEY,
                anchor,
                spend.body.balance_commitment,
                spend.body.nullifier,
                spend.body.rk,
            )
            .context("a spend proof did not verify")?;

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        // Check that the `Nullifier` has not been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state.object_get("source").unwrap_or_default();

        state.spend_nullifier(self.body.nullifier, source).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use penumbra_chain::test_keys;
    use penumbra_crypto::{transaction::Fee, Note, Value, STAKING_TOKEN_ASSET_ID};
    use penumbra_tct as tct;
    use penumbra_transaction::{
        plan::{OutputPlan, SpendPlan, TransactionPlan},
        WitnessData,
    };
    use rand_core::OsRng;

    use crate::ActionHandler;

    #[tokio::test]
    async fn check_stateless_succeeds_on_valid_spend() -> Result<()> {
        // Generate two notes controlled by the test address.
        let value = Value {
            amount: 100u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note = Note::generate(&mut OsRng, &*test_keys::ADDRESS_0, value);
        let value2 = Value {
            amount: 50u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note2 = Note::generate(&mut OsRng, &*test_keys::ADDRESS_0, value2);

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        // Assume there's a bunch of stuff already in the SCT.
        for _ in 0..5 {
            let random_note = Note::generate(&mut OsRng, &*test_keys::ADDRESS_0, value);
            sct.insert(tct::Witness::Keep, random_note.commit())
                .unwrap();
        }
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        sct.insert(tct::Witness::Keep, note2.commit()).unwrap();
        // Do we want to seal the SCT block here?
        let auth_path = sct.witness(note.commit()).unwrap();
        let auth_path2 = sct.witness(note2.commit()).unwrap();

        // Add a single spend and output to the transaction plan such that the
        // transaction balances.
        let plan = TransactionPlan {
            expiry_height: 0,
            fee: Fee::default(),
            chain_id: "".into(),
            actions: vec![
                SpendPlan::new(&mut OsRng, note, auth_path.position()).into(),
                SpendPlan::new(&mut OsRng, note2, auth_path2.position()).into(),
                OutputPlan::new(&mut OsRng, value, *test_keys::ADDRESS_1).into(),
            ],
            clue_plans: vec![],
            memo_plan: None,
        };

        // Build the transaction.
        let fvk = &test_keys::FULL_VIEWING_KEY;
        let sk = &test_keys::SPEND_KEY;
        let auth_data = plan.authorize(OsRng, &sk);
        let witness_data = WitnessData {
            anchor: sct.root(),
            state_commitment_proofs: plan
                .spend_plans()
                .map(|spend| {
                    (
                        spend.note.commit(),
                        sct.witness(spend.note.commit()).unwrap(),
                    )
                })
                .collect(),
        };
        let mut rng = OsRng;
        let tx = plan
            .build_concurrent(&mut rng, fvk, witness_data)
            .await
            .expect("can build transaction")
            .authorize(&mut rng, &auth_data)
            .expect("can authorize transaction");

        let context = Arc::new(tx.clone());

        // On the verifier side, perform stateless verification.
        for action in tx.transaction_body().actions {
            let result = action.check_stateless(context.clone()).await;
            assert!(result.is_ok())
        }

        Ok(())
    }

    #[tokio::test]
    async fn check_stateless_fails_on_auth_path_with_wrong_root() -> Result<()> {
        // Generate a note controlled by the test address.
        let value = Value {
            amount: 100u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note = Note::generate(&mut OsRng, &*test_keys::ADDRESS_0, value);

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        let wrong_root = sct.root();
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        let auth_path = sct.witness(note.commit()).unwrap();

        // Add a single spend and output to the transaction plan such that the
        // transaction balances.
        let plan = TransactionPlan {
            expiry_height: 0,
            fee: Fee::default(),
            chain_id: "".into(),
            actions: vec![
                SpendPlan::new(&mut OsRng, note, auth_path.position()).into(),
                OutputPlan::new(&mut OsRng, value, *test_keys::ADDRESS_1).into(),
            ],
            clue_plans: vec![],
            memo_plan: None,
        };

        // Build the transaction.
        let fvk = &test_keys::FULL_VIEWING_KEY;
        let sk = &test_keys::SPEND_KEY;
        let auth_data = plan.authorize(OsRng, &sk);
        let witness_data = WitnessData {
            anchor: sct.root(),
            state_commitment_proofs: plan
                .spend_plans()
                .map(|spend| {
                    (
                        spend.note.commit(),
                        sct.witness(spend.note.commit()).unwrap(),
                    )
                })
                .collect(),
        };
        let mut rng = OsRng;
        let mut tx = plan
            .build_concurrent(&mut rng, fvk, witness_data)
            .await
            .expect("can build transaction")
            .authorize(&mut rng, &auth_data)
            .expect("can authorize transaction");

        // Set the anchor to the wrong root.
        tx.anchor = wrong_root;

        let context = Arc::new(tx.clone());
        // On the verifier side, perform stateless verification.
        let result = tx.check_stateless(context.clone()).await;
        assert!(result.is_err());

        Ok(())
    }
}
