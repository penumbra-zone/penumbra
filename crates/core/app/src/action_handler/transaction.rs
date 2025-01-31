use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_fee::component::FeePay as _;
use penumbra_sdk_sct::component::source::SourceContext;
use penumbra_sdk_shielded_pool::component::ClueManager;
use penumbra_sdk_transaction::{gas::GasCost as _, Transaction};
use tokio::task::JoinSet;
use tracing::{instrument, Instrument};

use super::AppActionHandler;

mod stateful;
mod stateless;

use self::stateful::{
    claimed_anchor_is_valid, fmd_parameters_valid, tx_parameters_historical_check,
};
use stateless::{
    check_memo_exists_if_outputs_absent_if_not, check_non_empty_transaction,
    num_clues_equal_to_num_outputs, valid_binding_signature,
};

#[async_trait]
impl AppActionHandler for Transaction {
    type CheckStatelessContext = ();

    // We only instrument the top-level `check_stateless`, so we get one span for each transaction.
    #[instrument(skip(self, _context))]
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // This check should be done first, and complete before all other
        // stateless checks, like proof verification.  In addition to proving
        // that value balances, the binding signature binds the proofs to the
        // transaction, as the binding signature can only be created with
        // knowledge of all of the openings to the commitments the transaction
        // makes proofs against. (This is where the name binding signature comes
        // from).
        //
        // This allows us to cheaply eliminate a large class of invalid
        // transactions upfront -- past this point, we can be sure that the user
        // who submitted the transaction actually formed the proofs, rather than
        // replaying them from another transaction.
        valid_binding_signature(self)?;
        // Other checks probably too cheap to be worth splitting into tasks.
        num_clues_equal_to_num_outputs(self)?;
        check_memo_exists_if_outputs_absent_if_not(self)?;
        // This check ensures that transactions contain at least one action.
        check_non_empty_transaction(self)?;

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
    async fn check_historical<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let mut action_checks = JoinSet::new();

        // SAFETY: Transaction parameters (chain id, expiry height) against chain state
        // that cannot change during transaction execution.
        // The fee is _not_ checked here, but during execution.
        tx_parameters_historical_check(state.clone(), self).await?;
        // SAFETY: anchors are historical data and cannot change during transaction execution.
        claimed_anchor_is_valid(state.clone(), self).await?;
        // SAFETY: FMD parameters cannot change during transaction execution.
        fmd_parameters_valid(state.clone(), self).await?;

        // Currently, we need to clone the component actions so that the spawned
        // futures can have 'static lifetimes. In the future, we could try to
        // use the yoke crate, but cloning is almost certainly not a big deal
        // for now.
        for (i, action) in self.actions().cloned().enumerate() {
            let state2 = state.clone();
            let span = action.create_span(i);
            action_checks
                .spawn(async move { action.check_historical(state2).await }.instrument(span));
        }
        // Now check if any component action failed verification.
        while let Some(check) = action_checks.join_next().await {
            check??;
        }

        Ok(())
    }

    // We only instrument the top-level `execute`, so we get one span for each transaction.
    #[instrument(skip(self, state))]
    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // While we have access to the full Transaction, hash it to
        // obtain a NoteSource we can cache for various actions.
        state.put_current_source(Some(self.id()));

        // Check and record the transaction's fee payment,
        // before doing the rest of execution.
        let gas_used = self.gas_cost();
        let fee = self.transaction_body.transaction_parameters.fee;
        state.pay_fee(gas_used, fee).await?;

        for (i, action) in self.actions().enumerate() {
            let span = action.create_span(i);
            action
                .check_and_execute(&mut state)
                .instrument(span)
                .await?;
        }

        // Delete the note source, in case someone else tries to read it.
        state.put_current_source(None);

        // Record all the clues in this transaction
        // To avoid recomputing a hash.
        let id = self.id();
        for clue in self
            .transaction_body
            .detection_data
            .iter()
            .flat_map(|x| x.fmd_clues.iter())
        {
            state.record_clue(clue.clone(), id.clone()).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use anyhow::Result;
    use penumbra_sdk_asset::{Value, STAKING_TOKEN_ASSET_ID};
    use penumbra_sdk_fee::Fee;
    use penumbra_sdk_keys::test_keys;
    use penumbra_sdk_shielded_pool::{Note, OutputPlan, SpendPlan};
    use penumbra_sdk_tct as tct;
    use penumbra_sdk_transaction::{
        plan::{CluePlan, DetectionDataPlan, TransactionPlan},
        TransactionParameters, WitnessData,
    };
    use rand_core::OsRng;

    use crate::AppActionHandler;

    #[tokio::test]
    async fn check_stateless_succeeds_on_valid_spend() -> Result<()> {
        // Generate two notes controlled by the test address.
        let value = Value {
            amount: 100u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);
        let value2 = Value {
            amount: 50u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note2 = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value2);

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        // Assume there's a bunch of stuff already in the SCT.
        for _ in 0..5 {
            let random_note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);
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
            transaction_parameters: TransactionParameters {
                expiry_height: 0,
                fee: Fee::default(),
                chain_id: "".into(),
            },
            actions: vec![
                SpendPlan::new(&mut OsRng, note, auth_path.position()).into(),
                SpendPlan::new(&mut OsRng, note2, auth_path2.position()).into(),
                OutputPlan::new(&mut OsRng, value, test_keys::ADDRESS_1.deref().clone()).into(),
            ],
            detection_data: Some(DetectionDataPlan {
                clue_plans: vec![CluePlan::new(
                    &mut OsRng,
                    test_keys::ADDRESS_1.deref().clone(),
                    1.try_into().unwrap(),
                )],
            }),
            memo: None,
        };

        // Build the transaction.
        let fvk = &test_keys::FULL_VIEWING_KEY;
        let sk = &test_keys::SPEND_KEY;
        let auth_data = plan.authorize(OsRng, sk)?;
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
        let tx = plan
            .build_concurrent(fvk, &witness_data, &auth_data)
            .await
            .expect("can build transaction");

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
        let note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        let wrong_root = sct.root();
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        let auth_path = sct.witness(note.commit()).unwrap();

        // Add a single spend and output to the transaction plan such that the
        // transaction balances.
        let plan = TransactionPlan {
            transaction_parameters: TransactionParameters {
                expiry_height: 0,
                fee: Fee::default(),
                chain_id: "".into(),
            },
            actions: vec![
                SpendPlan::new(&mut OsRng, note, auth_path.position()).into(),
                OutputPlan::new(&mut OsRng, value, test_keys::ADDRESS_1.deref().clone()).into(),
            ],
            detection_data: None,
            memo: None,
        };

        // Build the transaction.
        let fvk = &test_keys::FULL_VIEWING_KEY;
        let sk = &test_keys::SPEND_KEY;
        let auth_data = plan.authorize(OsRng, sk)?;
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
        let mut tx = plan
            .build_concurrent(fvk, &witness_data, &auth_data)
            .await
            .expect("can build transaction");

        // Set the anchor to the wrong root.
        tx.anchor = wrong_root;

        // On the verifier side, perform stateless verification.
        let result = tx.check_stateless(()).await;
        assert!(result.is_err());

        Ok(())
    }
}
