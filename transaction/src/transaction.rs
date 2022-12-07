use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use anyhow::Error;
use ark_ff::Zero;
use bytes::Bytes;
use decaf377_fmd::Clue;
use penumbra_crypto::{
    memo::MemoCiphertext,
    note::Commitment,
    rdsa::{Binding, Signature, VerificationKey, VerificationKeyBytes},
    transaction::Fee,
    EncryptedNote, Fr, FullViewingKey, Note, Nullifier, PayloadKey,
};
use penumbra_proto::{
    core::ibc::v1alpha1 as pb_ibc, core::stake::v1alpha1 as pbs,
    core::transaction::v1alpha1 as pbt, Message, Protobuf,
};
use penumbra_tct as tct;
use serde::{Deserialize, Serialize};

use crate::{
    action::{Delegate, Output, ProposalSubmit, ProposalWithdraw, Swap, Undelegate, ValidatorVote},
    view::action_view::OutputView,
    Action, ActionView, IsAction, TransactionPerspective, TransactionView,
};

#[derive(Clone, Debug, Default)]
pub struct TransactionBody {
    pub actions: Vec<Action>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
    pub fmd_clues: Vec<Clue>,
    pub memo: Option<MemoCiphertext>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::Transaction", into = "pbt::Transaction")]
pub struct Transaction {
    pub transaction_body: TransactionBody,
    pub binding_sig: Signature<Binding>,
    pub anchor: tct::Root,
}

impl Default for Transaction {
    fn default() -> Self {
        Transaction {
            transaction_body: Default::default(),
            binding_sig: [0u8; 64].into(),
            anchor: tct::Tree::new().root(),
        }
    }
}

impl Transaction {
    pub fn payload_keys(
        &self,
        fvk: &FullViewingKey,
    ) -> anyhow::Result<BTreeMap<Commitment, PayloadKey>> {
        let mut result = BTreeMap::new();

        for action in self.actions() {
            match action {
                Action::Swap(swap) => {
                    let epk = &swap.body.swap_nft.ephemeral_key;
                    let shared_secret = fvk.incoming().key_agreement_with(epk)?;
                    let payload_key = PayloadKey::derive(&shared_secret, epk);
                    let commitment = swap.body.swap_nft.note_commitment;

                    result.insert(commitment, payload_key);
                }
                Action::SwapClaim(swap_claim) => {
                    let epk_1 = &swap_claim.body.output_1.ephemeral_key;
                    let epk_2 = &swap_claim.body.output_2.ephemeral_key;
                    let shared_secret_1 = fvk.incoming().key_agreement_with(epk_1)?;
                    let shared_secret_2 = fvk.incoming().key_agreement_with(epk_2)?;
                    let payload_key_1 = PayloadKey::derive(&shared_secret_1, epk_1);
                    let payload_key_2 = PayloadKey::derive(&shared_secret_2, epk_2);
                    let commitment_1 = swap_claim.body.output_1.note_commitment;
                    let commitment_2 = swap_claim.body.output_2.note_commitment;

                    result.insert(commitment_1, payload_key_1);
                    result.insert(commitment_2, payload_key_2);
                }
                Action::Output(output) => {
                    // Outputs may be either incoming or outgoing; for an outgoing output
                    // we need to use the ovk_wrapped_key, and for an incoming output we need to
                    // use the IVK to perform key agreement with the ephemeral key.
                    let ovk_wrapped_key = output.body.ovk_wrapped_key.clone();
                    let commitment = output.body.note_payload.note_commitment;
                    let epk = &output.body.note_payload.ephemeral_key;
                    let cv = output.body.balance_commitment;
                    let ovk = fvk.outgoing();
                    let shared_secret =
                        Note::decrypt_key(ovk_wrapped_key, commitment, cv, ovk, epk);

                    match shared_secret {
                        Ok(shared_secret) => {
                            // This is an outgoing output.
                            let payload_key = PayloadKey::derive(&shared_secret, epk);
                            result.insert(commitment, payload_key);
                        }
                        Err(_) => {
                            // This is (maybe) an incoming output, use the ivk.
                            let shared_secret = fvk.incoming().key_agreement_with(epk)?;
                            let payload_key = PayloadKey::derive(&shared_secret, epk);

                            result.insert(commitment, payload_key);
                        }
                    }
                }
                // These actions have no payload keys; they're listed explicitly
                // for exhaustiveness.
                Action::Spend(_) => {}
                Action::Delegate(_) => {}
                Action::Undelegate(_) => {}
                Action::UndelegateClaim(_) => {}
                Action::ValidatorDefinition(_) => {}
                Action::IBCAction(_) => {}
                Action::ProposalSubmit(_) => {}
                Action::ProposalWithdraw(_) => {}
                Action::ValidatorVote(_) => {}
                Action::PositionOpen(_) => {}
                Action::PositionClose(_) => {}
                Action::PositionWithdraw(_) => {}
                Action::PositionRewardClaim(_) => {}
                Action::Ics20Withdrawal(_) => {}
            }
        }

        Ok(result)
    }
    pub fn decrypt_with_perspective(&self, txp: &TransactionPerspective) -> TransactionView {
        let mut action_views = Vec::new();

        let mut memo_plaintext: Option<String> = None;

        for action in self.actions() {
            let action_view = action.view_from_perspective(txp);

            // In the case of Output actions, decrypt the transaction memo if this hasn't already been done.
            if let ActionView::Output(output) = &action_view {
                if memo_plaintext.is_none() {
                    memo_plaintext = match self.transaction_body().memo {
                        Some(ciphertext) => match output {
                            OutputView::Visible {
                                output: _,
                                note: _,
                                payload_key: decrypted_memo_key,
                            } => MemoCiphertext::decrypt(decrypted_memo_key, ciphertext).ok(),
                            OutputView::Opaque { output: _ } => None,
                        },
                        None => None,
                    }
                }
            }

            action_views.push(action_view);
        }

        TransactionView {
            action_views,
            expiry_height: self.transaction_body().expiry_height,
            chain_id: self.transaction_body().chain_id,
            fee: self.transaction_body().fee,
            fmd_clues: self.transaction_body().fmd_clues,
            memo: memo_plaintext,
        }
    }

    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.transaction_body.actions.iter()
    }

    pub fn delegations(&self) -> impl Iterator<Item = &Delegate> {
        self.actions().filter_map(|action| {
            if let Action::Delegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn undelegations(&self) -> impl Iterator<Item = &Undelegate> {
        self.actions().filter_map(|action| {
            if let Action::Undelegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn proposal_submits(&self) -> impl Iterator<Item = &ProposalSubmit> {
        self.actions().filter_map(|action| {
            if let Action::ProposalSubmit(s) = action {
                Some(s)
            } else {
                None
            }
        })
    }

    pub fn proposal_withdraws(&self) -> impl Iterator<Item = &ProposalWithdraw> {
        self.actions().filter_map(|action| {
            if let Action::ProposalWithdraw(w) = action {
                Some(w)
            } else {
                None
            }
        })
    }

    pub fn validator_votes(&self) -> impl Iterator<Item = &ValidatorVote> {
        self.actions().filter_map(|action| {
            if let Action::ValidatorVote(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    // pub fn delegator_votes(&self) -> impl Iterator<Item = &DelegatorVote> {
    //     self.actions().filter_map(|action| {
    //         if let Action::DelegatorVote(v) = action {
    //             Some(v)
    //         } else {
    //             None
    //         }
    //     })
    // }

    pub fn ibc_actions(&self) -> impl Iterator<Item = &pb_ibc::IbcAction> {
        self.actions().filter_map(|action| {
            if let Action::IBCAction(ibc_action) = action {
                Some(ibc_action)
            } else {
                None
            }
        })
    }

    pub fn validator_definitions(&self) -> impl Iterator<Item = &pbs::ValidatorDefinition> {
        self.actions().filter_map(|action| {
            if let Action::ValidatorDefinition(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn outputs(&self) -> impl Iterator<Item = &Output> {
        self.actions().filter_map(|action| {
            if let Action::Output(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn swaps(&self) -> impl Iterator<Item = &Swap> {
        self.actions().filter_map(|action| {
            if let Action::Swap(s) = action {
                Some(s)
            } else {
                None
            }
        })
    }

    // TODO: re-evaluate this API, do we want to be iterating over encrypted notes?
    // or state payloads? but the state payloads have sources, so we'd have to construct them
    // on the fly with the tx hash? how do we really want to be using tihs?
    pub fn encrypted_notes(&self) -> impl Iterator<Item = &EncryptedNote> {
        // This is somewhat cursed but avoids the need to allocate or erase types, I guess?
        self.actions()
            .flat_map(|action| match action {
                Action::Output(output) => [Some(&output.body.note_payload), None],
                Action::Swap(swap) => [Some(&swap.body.swap_nft), None],
                Action::SwapClaim(swap_claim) => [
                    Some(&swap_claim.body.output_1),
                    Some(&swap_claim.body.output_2),
                ],
                _ => [None, None],
            })
            // We've padded arrays with None to be able to unify types, now strip the
            // bogus padding values away:
            .flatten()
    }

    pub fn spent_nullifiers(&self) -> impl Iterator<Item = Nullifier> + '_ {
        self.actions().filter_map(|action| {
            // Note: adding future actions that include nullifiers
            // will need to be matched here as well as Spends
            match action {
                Action::Spend(spend) => Some(spend.body.nullifier),
                Action::SwapClaim(swap_claim) => Some(swap_claim.body.nullifier),
                _ => None,
            }
        })
    }

    pub fn transaction_body(&self) -> TransactionBody {
        self.transaction_body.clone()
    }

    pub fn binding_sig(&self) -> &Signature<Binding> {
        &self.binding_sig
    }

    pub fn id(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let tx_bytes: Vec<u8> = self.clone().try_into().expect("can serialize transaction");
        let mut id_bytes = [0; 32];
        id_bytes[..].copy_from_slice(Sha256::digest(&tx_bytes).as_slice());

        id_bytes
    }

    /// Compute the binding verification key from the transaction data.
    pub fn binding_verification_key(&self) -> VerificationKey<Binding> {
        let mut balance_commitments = decaf377::Element::default();
        for action in &self.transaction_body.actions {
            balance_commitments += action.balance_commitment().0;
        }

        // Add fee into binding verification key computation.
        let fee_v_blinding = Fr::zero();
        let fee_value_commitment = self.transaction_body.fee.commit(fee_v_blinding);
        balance_commitments += fee_value_commitment.0;

        let binding_verification_key_bytes: VerificationKeyBytes<Binding> =
            balance_commitments.vartime_compress().0.into();

        binding_verification_key_bytes
            .try_into()
            .expect("verification key is valid")
    }
}

impl From<TransactionBody> for Vec<u8> {
    fn from(transaction_body: TransactionBody) -> Vec<u8> {
        let protobuf_serialized: pbt::TransactionBody = transaction_body.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl Protobuf<pbt::TransactionBody> for TransactionBody {}

impl From<TransactionBody> for pbt::TransactionBody {
    fn from(msg: TransactionBody) -> Self {
        pbt::TransactionBody {
            actions: msg.actions.into_iter().map(|x| x.into()).collect(),
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
            fee: Some(msg.fee.into()),
            fmd_clues: msg.fmd_clues.into_iter().map(|x| x.into()).collect(),
            encrypted_memo: msg.memo.map(|x| bytes::Bytes::copy_from_slice(&x.0)),
        }
    }
}

impl TryFrom<pbt::TransactionBody> for TransactionBody {
    type Error = Error;

    fn try_from(proto: pbt::TransactionBody) -> anyhow::Result<Self, Self::Error> {
        let mut actions = Vec::<Action>::new();
        for action in proto.actions {
            actions.push(
                action
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("transaction body malformed"))?,
            );
        }

        let expiry_height = proto.expiry_height;

        let chain_id = proto.chain_id;

        let fee: Fee = proto
            .fee
            .ok_or_else(|| anyhow::anyhow!("transaction body malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("clue malformed"))?;

        let mut fmd_clues = Vec::<Clue>::new();
        for fmd_clue in proto.fmd_clues {
            fmd_clues.push(
                fmd_clue
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("clue malformed"))?,
            );
        }

        let memo = match proto.encrypted_memo {
            Some(bytes) => Some(
                bytes[..]
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("memo malformed"))?,
            ),
            None => None,
        };

        Ok(TransactionBody {
            actions,
            expiry_height,
            chain_id,
            fee,
            fmd_clues,
            memo,
        })
    }
}
impl Protobuf<pbt::Transaction> for Transaction {}

impl From<Transaction> for pbt::Transaction {
    fn from(msg: Transaction) -> Self {
        let sig_bytes: [u8; 64] = msg.binding_sig.into();
        pbt::Transaction {
            body: Some(msg.transaction_body.into()),
            anchor: Some(msg.anchor.into()),
            binding_sig: Bytes::copy_from_slice(&sig_bytes),
        }
    }
}

impl From<&Transaction> for pbt::Transaction {
    fn from(msg: &Transaction) -> Self {
        msg.into()
    }
}

impl TryFrom<pbt::Transaction> for Transaction {
    type Error = Error;

    fn try_from(proto: pbt::Transaction) -> anyhow::Result<Self, Self::Error> {
        let transaction_body = proto
            .body
            .ok_or_else(|| anyhow::anyhow!("transaction malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction body malformed"))?;

        let sig_bytes: [u8; 64] = proto.binding_sig[..]
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction malformed"))?;

        let anchor = proto
            .anchor
            .ok_or_else(|| anyhow::anyhow!("transaction malformed"))?
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction malformed"))?;

        Ok(Transaction {
            transaction_body,
            binding_sig: sig_bytes.into(),
            anchor,
        })
    }
}

impl TryFrom<&[u8]> for Transaction {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Transaction, Self::Error> {
        let protobuf_serialized_proof = pbt::Transaction::decode(bytes)
            .map_err(|_| anyhow::anyhow!("transaction malformed"))?;
        protobuf_serialized_proof
            .try_into()
            .map_err(|_| anyhow::anyhow!("transaction malformed"))
    }
}

impl TryFrom<Vec<u8>> for Transaction {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Transaction, Self::Error> {
        Self::try_from(&bytes[..])
    }
}

impl From<Transaction> for Vec<u8> {
    fn from(transaction: Transaction) -> Vec<u8> {
        let protobuf_serialized: pbt::Transaction = transaction.into();
        protobuf_serialized.encode_to_vec()
    }
}

impl From<&Transaction> for Vec<u8> {
    fn from(transaction: &Transaction) -> Vec<u8> {
        let protobuf_serialized: pbt::Transaction = transaction.into();
        protobuf_serialized.encode_to_vec()
    }
}
