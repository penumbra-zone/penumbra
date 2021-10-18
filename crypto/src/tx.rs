use ark_ff::UniformRand;
use rand::seq::SliceRandom;
use rand_core::{CryptoRng, RngCore};

use crate::{
    action::{output, spend},
    addresses::PaymentAddress,
    asset,
    keys::{OutgoingViewingKey, SpendKey},
    memo::MemoPlaintext,
    merkle::proof::MerklePath,
    nullifier::Nullifier,
    Fr, Note, Output, Spend, Value,
};

/// Used to construct a Penumbra transaction.
pub struct TransactionBuilder {
    // Notes we'll consume in this transaction.
    spends: Vec<Spend>,
    // Notes we'll create in this transaction.
    outputs: Vec<Output>,
    // Transaction fee. None if unset.
    fee: Option<u64>,
    // Total value changed in this transaction.
    balance: i64,
    // Put chain_id and anchor in here too?
}

impl TransactionBuilder {
    /// Create a new `Spend` to spend an existing note.
    pub fn add_spend<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        spend_key: SpendKey,
        _merkle_path: MerklePath,
        note: Note,
    ) {
        self.balance -= note.value.amount as i64;

        // TODO: Derive nullifier from note commitment, note position, and
        // nullifier deriving key
        // See p.55 ZCash spec
        let nullifier = Nullifier::new();

        let v_blinding = Fr::rand(rng);
        let value_commitment = note.value.commit(v_blinding);

        let spend_auth_randomizer = Fr::rand(rng);
        let rsk = spend_key.spend_auth_key().randomize(&spend_auth_randomizer);

        let body = spend::Body::new(
            rng,
            value_commitment,
            nullifier,
            *spend_key.spend_auth_key(),
            spend_auth_randomizer,
        );

        let auth_sig = rsk.sign(rng, &body.serialize());

        let spend = Spend { body, auth_sig };

        self.spends.push(spend);
    }

    /// Create a new `Output` to create a new note.
    pub fn add_output<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
        dest: &PaymentAddress,
        asset_id: asset::Id,
        amount: u64,
        memo: MemoPlaintext,
        _ovk: &OutgoingViewingKey,
    ) {
        let value_to_send = Value { amount, asset_id };
        self.balance += value_to_send.amount as i64;

        let body = output::Body::new(rng, value_to_send, dest);

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
    }

    /// Set the transaction fee in PEN.
    pub fn set_fee(&mut self, fee: u64) {
        self.balance -= fee as i64;
        self.fee = Some(fee)
    }

    // xx Uses rand::RngCore instead of RngCore
    pub fn finalize<R: CryptoRng + rand::RngCore + rand::seq::SliceRandom>(
        &mut self,
        rng: &mut R,
    ) -> TransactionBody {
        // Randomize outputs to minimize info leakage.
        self.outputs.shuffle(rng);
        self.spends.shuffle(rng);

        // Apply sig
        todo!();
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self {
            spends: Vec::<Spend>::new(),
            outputs: Vec::<Output>::new(),
            fee: None,
            // xx Per asset?
            balance: 0,
        }
    }
}

pub struct TransactionBody {}

impl TransactionBody {
    pub fn sign() -> Transaction {
        todo!()
    }
}

pub struct Transaction {}

impl Transaction {
    pub fn builder() -> TransactionBuilder {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::keys::Diversifier;
    use rand_core::OsRng;

    // Not really a test - just to exercise the code path for now
    #[test]
    fn test_transaction_create() {
        let mut rng = OsRng;
        let sk_sender = SpendKey::generate(&mut rng);
        let fvk_sender = sk_sender.full_viewing_key();
        let ivk_sender = fvk_sender.outgoing();

        let sk_recipient = SpendKey::generate(&mut rng);
        let diversifier_recipient = Diversifier::generate(&mut rng);
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let dest = PaymentAddress::new(ivk_recipient, diversifier_recipient);

        let mut builder = Transaction::builder();
        builder.set_fee(20);

        let pen_trace = b"pen";
        let pen_id = asset::Id::from(&pen_trace[..]);
        let memo = MemoPlaintext::default();
        builder.add_output(&mut rng, &dest, pen_id, 10, memo, ivk_sender);
        builder.set_fee(10);
    }
}
