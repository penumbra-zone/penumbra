use penumbra_crypto::{balance, Fr, Note, Zero};
use penumbra_dao::{DaoDeposit, DaoOutput, DaoSpend};
use penumbra_ibc::{IbcAction, Ics20Withdrawal};
use penumbra_shielded_pool::{Output, OutputView, Spend, SpendView};

use crate::{ActionView, TransactionPerspective};

// TODO: how do we have this be implemented in the component crates?
// currently can't because of txp

/// Common behavior between Penumbra actions.
pub trait IsAction {
    fn balance_commitment(&self) -> balance::Commitment;
    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView;
}

// foreign types

impl IsAction for Output {
    fn balance_commitment(&self) -> balance::Commitment {
        self.body.balance_commitment
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        let note_commitment = self.body.note_payload.note_commitment;
        let epk = self.body.note_payload.ephemeral_key;
        // Retrieve payload key for associated note commitment
        let output_view = if let Some(payload_key) = txp.payload_keys.get(&note_commitment) {
            let decrypted_note = Note::decrypt_with_payload_key(
                &self.body.note_payload.encrypted_note,
                payload_key,
                &epk,
            );

            let decrypted_memo_key = self.body.wrapped_memo_key.decrypt_outgoing(payload_key);

            if let (Ok(decrypted_note), Ok(decrypted_memo_key)) =
                (decrypted_note, decrypted_memo_key)
            {
                // Neither decryption failed, so return the visible ActionView
                OutputView::Visible {
                    output: self.to_owned(),
                    note: txp.view_note(decrypted_note),
                    payload_key: decrypted_memo_key,
                }
            } else {
                // One or both of the note or memo key is missing, so return the opaque ActionView
                OutputView::Opaque {
                    output: self.to_owned(),
                }
            }
        } else {
            // There was no payload key found, so return the opaque ActionView
            OutputView::Opaque {
                output: self.to_owned(),
            }
        };

        ActionView::Output(output_view)
    }
}

impl IsAction for Spend {
    fn balance_commitment(&self) -> balance::Commitment {
        self.body.balance_commitment
    }

    fn view_from_perspective(&self, txp: &TransactionPerspective) -> ActionView {
        let spend_view = match txp.spend_nullifiers.get(&self.body.nullifier) {
            Some(note) => SpendView::Visible {
                spend: self.to_owned(),
                note: txp.view_note(note.to_owned()),
            },
            None => SpendView::Opaque {
                spend: self.to_owned(),
            },
        };

        ActionView::Spend(spend_view)
    }
}

impl IsAction for IbcAction {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        Default::default()
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::IbcAction(self.clone())
    }
}

impl IsAction for Ics20Withdrawal {
    fn balance_commitment(&self) -> penumbra_crypto::balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::Ics20Withdrawal(self.to_owned())
    }
}

impl IsAction for DaoDeposit {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::DaoDeposit(self.clone())
    }
}

impl IsAction for DaoOutput {
    fn balance_commitment(&self) -> balance::Commitment {
        // Outputs from the DAO require value
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::DaoOutput(self.clone())
    }
}

impl IsAction for DaoSpend {
    fn balance_commitment(&self) -> balance::Commitment {
        self.balance().commit(Fr::zero())
    }

    fn view_from_perspective(&self, _txp: &TransactionPerspective) -> ActionView {
        ActionView::DaoSpend(self.clone())
    }
}
