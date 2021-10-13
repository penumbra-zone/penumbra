use rand_core::{CryptoRng, RngCore};

use crate::{
    addresses::PaymentAddress,
    asset,
    keys::{Diversifier, OutgoingViewingKey, SpendKey},
    memo::MemoPlaintext,
    merkle::proof::MerklePath,
    Note, Output, Spend, Value,
};

/// Used to construct a Penumbra transaction.
pub struct TransactionBuilder {
    // Notes we'll consume in this transaction.
    spends: Vec<Spend>,
    // Notes we'll create in this transaction.
    outputs: Vec<Output>,
    // Transaction fee.
    fee: u32,
    // Total value changed in this transaction.
    balance: u32,
    // Put chain_id and anchor in here too?
}

// Idea TransactionBuilder.finalize() -> UnsignedTransaction
// UnsignedTransaction.sign -> Transaction --- this applies the binding_sig
// or Maybe directly TransactionBuilder.finalize() -> Transaction

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            spends: Vec::<Spend>::new(),
            outputs: Vec::<Output>::new(),
            fee: 0,
            balance: 0,
        }
    }

    /// Create a new `Spend` to spend an existing note.
    pub fn add_spend<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        diversifier: &Diversifier,
        spend_key: SpendKey,
        merkle_path: MerklePath,
        note: Note,
    ) {
        let spend = Spend::new(rng, diversifier, spend_key, merkle_path, note);
        self.spends.push(spend);
    }

    /// Create a new `Output` to create a new note.
    pub fn add_output<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        dest: &PaymentAddress,
        value: Value,
        memo: MemoPlaintext,
        ovk: &OutgoingViewingKey,
    ) {
        let output = Output::new(rng, dest, value, memo, ovk);
        self.outputs.push(output);
    }

    /// Set the transaction fee in PEN.
    pub fn set_fee(&mut self, fee: u32) {
        self.fee = fee
    }

    /// Add and subtract value commitments to compute transaction balance.
    pub fn compute_balance_check() -> Value {
        // Balance is in PEN?
        // Check all assets are the same?
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand_core::OsRng;

    // Not really a test - just to exercise the code path for now
    #[test]
    fn test_transaction_create() {
        let mut rng = OsRng;
        let diversifier = Diversifier::generate(&mut rng);
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let diversifier_recipient = Diversifier::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let dest = PaymentAddress::new(ivk_recipient, diversifier_recipient);

        let mut builder = TransactionBuilder::new();
        builder.set_fee(20);

        let pen_trace = b"pen";
        let pen_id = asset::Id::from(&pen_trace[..]);
        let value_to_send = Value {
            amount: 10,
            asset_id: pen_id,
        };
        let memo = MemoPlaintext::default();

        // xx Kind of awkward api, pass in ID and amount instead?
        builder.add_output(&mut rng, &dest, value_to_send, memo, ivk_sender);
    }
}
