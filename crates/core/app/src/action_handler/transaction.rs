use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use penumbra_chain::NoteSource;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;
use tokio::task::JoinSet;
use tracing::{instrument, Instrument};

use super::ActionHandler;

mod stateful;
mod stateless;

use self::stateful::{claimed_anchor_is_valid, fmd_parameters_valid};
use stateless::{
    check_memo_exists_if_outputs_absent_if_not, no_duplicate_nullifiers,
    num_clues_equal_to_num_outputs, valid_binding_signature,
};

#[async_trait]
impl ActionHandler for Transaction {
    type CheckStatelessContext = ();

    // We only instrument the top-level `check_stateless`, so we get one span for each transaction.
    #[instrument(skip(self, _context))]
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // TODO: add a check that ephemeral_key is not identity to prevent scanning dos attack ?

        // TODO: unify code organization
        valid_binding_signature(self)?;
        no_duplicate_nullifiers(self)?;
        num_clues_equal_to_num_outputs(self)?;
        check_memo_exists_if_outputs_absent_if_not(self)?;

        let context = self.context();

        // Currently, we need to clone the component actions so that the spawned
        // futures can have 'static lifetimes. In the future, we could try to
        // use the yoke crate, but cloning is almost certainly not a big deal
        // for now.
        let mut action_checks = JoinSet::new();
        for (i, action) in self.actions().cloned().enumerate() {
            let context2 = context.clone();
            let span = action.create_span(i);
            action_checks
                .spawn(async move { action.check_stateless(context2).await }.instrument(span));
        }
        // Now check if any component action failed verification.
        while let Some(check) = action_checks.join_next().await {
            check??;
        }

        Ok(())
    }

    // We only instrument the top-level `check_stateful`, so we get one span for each transaction.
    #[instrument(skip(self, state))]
    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        claimed_anchor_is_valid(state.clone(), self).await?;
        fmd_parameters_valid(state.clone(), self).await?;

        // Currently, we need to clone the component actions so that the spawned
        // futures can have 'static lifetimes. In the future, we could try to
        // use the yoke crate, but cloning is almost certainly not a big deal
        // for now.
        let mut action_checks = JoinSet::new();
        for (i, action) in self.actions().cloned().enumerate() {
            let state2 = state.clone();
            let span = action.create_span(i);
            action_checks
                .spawn(async move { action.check_stateful(state2).await }.instrument(span));
        }
        // Now check if any component action failed verification.
        while let Some(check) = action_checks.join_next().await {
            check??;
        }

        Ok(())
    }

    // We only instrument the top-level `execute`, so we get one span for each transaction.
    #[instrument(skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // While we have access to the full Transaction, hash it to
        // obtain a NoteSource we can cache for various actions.
        let source = NoteSource::Transaction { id: self.id().0 };
        state.object_put("source", source);

        for (i, action) in self.actions().enumerate() {
            let span = action.create_span(i);
            action.execute(&mut state).instrument(span).await?;
        }

        // Delete the note source, in case someone else tries to read it.
        state.object_delete("source");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use penumbra_asset::{Value, STAKING_TOKEN_ASSET_ID};
    use penumbra_chain::test_keys;
    use penumbra_fee::Fee;
    use penumbra_shielded_pool::{Note, OutputPlan, SpendPlan};
    use penumbra_tct as tct;
    use penumbra_transaction::{plan::TransactionPlan, WitnessData};
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

        let context = tx.context();

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

        // On the verifier side, perform stateless verification.
        let result = tx.check_stateless(()).await;
        assert!(result.is_err());

        Ok(())
    }
}
