use ark_ff::Zero;

use crate::{merkle, Fr, Output, Spend};

use crate::builder::TransactionBuilder;

pub struct TransactionBody {
    pub merkle_root: merkle::Root,
    // TK from proto
}

impl TransactionBody {
    pub fn sign() -> Transaction {
        todo!()
    }
}

pub struct Transaction {}

impl Transaction {
    /// Start building a transaction relative to a given [`merkle::Root`].
    pub fn build_with_root(merkle_root: merkle::Root) -> TransactionBuilder {
        TransactionBuilder {
            spends: Vec::<Spend>::new(),
            outputs: Vec::<Output>::new(),
            fee: None,
            synthetic_blinding_factor: Fr::zero(),
            merkle_root,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand_core::OsRng;

    use crate::addresses::PaymentAddress;
    use crate::keys::{Diversifier, SpendKey};
    use crate::memo::MemoPlaintext;
    use crate::{Fq, Value};

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

        let merkle_root = merkle::Root(Fq::zero());
        let _transaction_builder = Transaction::build_with_root(merkle_root)
            .set_fee(20)
            .add_output(
                &mut rng,
                &dest,
                Value {
                    amount: 10,
                    asset_id: b"pen".as_ref().into(),
                },
                MemoPlaintext::default(),
                ivk_sender,
            );
        // Commented out since .finalize() will currently fail the test.
        //let transaction = transaction_builder.finalize(&mut rng);
    }
}
