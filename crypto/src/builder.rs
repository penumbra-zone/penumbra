use ark_ff::UniformRand;
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};

use crate::{
    action::{output, spend},
    addresses::PaymentAddress,
    keys::{OutgoingViewingKey, SpendKey},
    memo::MemoPlaintext,
    merkle,
    nullifier::Nullifier,
    transaction::{Transaction, TransactionBody},
    Fr, Note, Output, Spend, Value,
};

/// Used to construct a Penumbra transaction.
pub struct TransactionBuilder {
    // Notes we'll consume in this transaction.
    pub spends: Vec<Spend>,
    // Notes we'll create in this transaction.
    pub outputs: Vec<Output>,
    // Transaction fee. None if unset.
    pub fee: Option<u64>,
    // Sum of blinding factors for each value commitment.
    pub synthetic_blinding_factor: Fr,
    // Put chain_id in here too?
    pub merkle_root: merkle::Root,
}

impl TransactionBuilder {
    /// Create a new `Spend` to spend an existing note.
    pub fn add_spend<R: RngCore + CryptoRng>(
        mut self,
        rng: &mut R,
        spend_key: SpendKey,
        merkle_path: merkle::Path,
        note: Note,
    ) -> Self {
        // TODO: Derive nullifier from note commitment, note position, and
        // nullifier deriving key
        // See p.55 ZCash spec
        let nullifier = Nullifier::new();

        let v_blinding = Fr::rand(rng);
        let value_commitment = note.value.commit(v_blinding);
        // We add to the transaction's value balance.
        self.synthetic_blinding_factor += v_blinding;

        let spend_auth_randomizer = Fr::rand(rng);
        let rsk = spend_key.spend_auth_key().randomize(&spend_auth_randomizer);

        let body = spend::Body::new(
            rng,
            value_commitment,
            nullifier,
            *spend_key.spend_auth_key(),
            spend_auth_randomizer,
            merkle_path,
        );

        let auth_sig = rsk.sign(rng, &body.serialize());

        let spend = Spend { body, auth_sig };

        self.spends.push(spend);

        self
    }

    /// Create a new `Output` to create a new note.
    pub fn add_output<R: RngCore + CryptoRng>(
        mut self,
        rng: &mut R,
        dest: &PaymentAddress,
        value_to_send: Value,
        memo: MemoPlaintext,
        _ovk: &OutgoingViewingKey,
    ) -> Self {
        let v_blinding = Fr::rand(rng);
        // We subtract from the transaction's value balance.
        self.synthetic_blinding_factor -= v_blinding;

        let body = output::Body::new(rng, value_to_send, v_blinding, dest);

        // Encrypted to receipient diversified payment addr?
        //let encrypted_memo = memo.encrypt(dest);
        // In Sapling, it seems like the memo field is encrypted as part of the
        // note, but in our protos we have the memo broken out.
        // TEMP: Transparent memos

        //let ovk_wrapped_key = todo!();

        let output = Output {
            body,
            memo,
            //encrypted_memo,
            // ovk_wrapped_key,
        };
        self.outputs.push(output);

        self
    }

    /// Set the transaction fee in PEN.
    pub fn set_fee(mut self, fee: u64) -> Self {
        self.fee = Some(fee);
        self
    }

    pub fn finalize<R: CryptoRng + RngCore>(mut self, rng: &mut R) -> Transaction {
        // Randomize outputs to minimize info leakage.
        self.outputs.shuffle(rng);
        self.spends.shuffle(rng);

        let _transaction_body = TransactionBody {
            merkle_root: self.merkle_root,
        };

        // Apply sig
        todo!();
    }
}
