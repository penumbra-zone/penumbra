use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use penumbra_proof_params::SPEND_PROOF_VERIFICATION_KEY;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::{action::Spend, Transaction};
use tracing::instrument;

use crate::{
    action_handler::ActionHandler,
    shielded_pool::{NoteManager, StateReadExt as _},
};

#[async_trait]
impl ActionHandler for Spend {
    #[instrument(name = "spend", skip(self, context))]
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

    #[instrument(name = "spend", skip(self, state))]
    async fn check_stateful<S: StateRead>(&self, state: Arc<S>) -> Result<()> {
        // Check that the `Nullifier` has not been spent before.
        let spent_nullifier = self.body.nullifier;
        state.check_nullifier_unspent(spent_nullifier).await
    }

    #[instrument(name = "spend", skip(self, state))]
    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let source = state.object_get("source").unwrap_or_default();

        state.spend_nullifier(self.body.nullifier, source).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use ark_ff::UniformRand;
    use decaf377::Fr;
    use penumbra_chain::test_keys;
    use penumbra_crypto::{
        asset,
        keys::{SeedPhrase, SpendKey},
        proofs::groth16::SpendProof,
        rdsa::{SpendAuth, VerificationKey},
        transaction::Fee,
        Note, Value, STAKING_TOKEN_ASSET_ID,
    };
    use penumbra_proof_params::{SPEND_PROOF_PROVING_KEY, SPEND_PROOF_VERIFICATION_KEY};
    use penumbra_tct as tct;
    use penumbra_transaction::plan::{SpendPlan, TransactionPlan};
    use rand_core::OsRng;

    #[tokio::test]
    async fn check_stateless_succeeds_on_valid_spend() -> Result<()> {
        let pk = &*SPEND_PROOF_PROVING_KEY;
        let vk = &*SPEND_PROOF_VERIFICATION_KEY;

        // Generate a note controlled by the test address.
        let note = Note::generate(
            &mut OsRng,
            &*test_keys::ADDRESS_0,
            Value {
                amount: 100u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        // Do we want to seal the SCT block here?
        let auth_path = sct.witness(note.commit()).unwrap();

        let plan = TransactionPlan {
            expiry_height: 0,
            fee: Fee::default(),
            chain_id: "".into(),
            actions: vec![SpendPlan::new(&mut OsRng, note, auth_path.position()).into()],
            clue_plans: vec![],
            memo_plan: None,
        };

        // Comp

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_sender = SpendKey::from_seed_phrase(seed_phrase, 0);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.incoming();
        let (sender, _dtk_d) = ivk_sender.payment_address(0u64.into());

        let value_to_send = Value {
            amount: 1u64.into(),
            asset_id: asset::REGISTRY.parse_denom("upenumbra").unwrap().id(),
        };

        let note = Note::generate(&mut OsRng, &sender, value_to_send);
        let note_commitment = note.commit();
        let spend_auth_randomizer = Fr::rand(&mut OsRng);
        let rsk = sk_sender.spend_auth_key().randomize(&spend_auth_randomizer);
        let nk = *sk_sender.nullifier_key();
        let ak: VerificationKey<SpendAuth> = sk_sender.spend_auth_key().into();
        let mut nct = tct::Tree::new();
        nct.insert(tct::Witness::Keep, note_commitment).unwrap();
        let anchor = nct.root();
        let note_commitment_proof = nct.witness(note_commitment).unwrap();
        let v_blinding = Fr::rand(&mut OsRng);
        let balance_commitment = value_to_send.commit(v_blinding);
        let rk: VerificationKey<SpendAuth> = rsk.into();
        let nf = nk.derive_nullifier(0.into(), &note_commitment);

        let proof = SpendProof::prove(
            &mut OsRng,
            pk,
            note_commitment_proof,
            note,
            v_blinding,
            spend_auth_randomizer,
            ak,
            nk,
            anchor,
            balance_commitment,
            nf,
            rk,
        )
        .expect("can create proof");

        let proof_result = proof.verify(vk, anchor, balance_commitment, nf, rk);
        assert!(proof_result.is_ok());
    }

    #[tokio::test]
    async fn check_stateless_fails_on_auth_sig_with_wrong_key() -> Result<()> {
        // TODO
        Ok(())
    }

    #[tokio::test]
    async fn check_stateless_fails_on_auth_sig_with_wrong_effect_hash() -> Result<()> {
        // TODO
        Ok(())
    }

    #[tokio::test]
    async fn check_stateless_fails_on_auth_path_with_wrong_root() -> Result<()> {
        // TODO
        Ok(())
    }
}
