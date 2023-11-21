use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

use anyhow::{Context, Error};
use ark_ff::Zero;
use decaf377::Fr;
use decaf377_fmd::Clue;
use decaf377_rdsa::{Binding, Signature, VerificationKey, VerificationKeyBytes};
use penumbra_chain::TransactionContext;
use penumbra_dao::{DaoDeposit, DaoOutput, DaoSpend};
use penumbra_dex::{
    lp::action::{PositionClose, PositionOpen},
    swap::Swap,
};
use penumbra_fee::Fee;
use penumbra_governance::{DelegatorVote, ProposalSubmit, ProposalWithdraw, ValidatorVote};
use penumbra_ibc::IbcRelay;
use penumbra_keys::{FullViewingKey, PayloadKey};
use penumbra_proto::{
    core::transaction::v1alpha1::{self as pbt},
    DomainType, Message,
};
use penumbra_sct::Nullifier;
use penumbra_shielded_pool::{Note, Output, Spend};
use penumbra_stake::{Delegate, Undelegate, UndelegateClaim};
use penumbra_tct as tct;
use penumbra_tct::StateCommitment;
use serde::{Deserialize, Serialize};

use crate::{
    memo::{MemoCiphertext, MemoPlaintext},
    view::{action_view::OutputView, MemoView, TransactionBodyView},
    Action, ActionView, Id, IsAction, MemoPlaintextView, TransactionPerspective, TransactionView,
};

#[derive(Clone, Debug, Default)]
pub struct TransactionBody {
    pub actions: Vec<Action>,
    pub transaction_parameters: TransactionParameters,
    pub fee: Fee,
    pub detection_data: Option<DetectionData>,
    pub memo: Option<MemoCiphertext>,
}

#[derive(Clone, Debug, Default)]
/// Parameters determining when the transaction should be accepted to the chain.
pub struct TransactionParameters {
    pub expiry_height: u64,
    pub chain_id: String,
}

#[derive(Clone, Debug, Default)]
/// Detection data used by a detection server using Fuzzy Message Detection.
///
/// Only present if outputs are present.
pub struct DetectionData {
    pub fmd_clues: Vec<Clue>,
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
    pub fn context(&self) -> TransactionContext {
        TransactionContext {
            anchor: self.anchor,
            effect_hash: self.effect_hash(),
        }
    }

    pub fn num_proofs(&self) -> usize {
        self.transaction_body
            .actions
            .iter()
            .map(|action| match action {
                Action::Spend(_) => 1,
                Action::Output(_) => 1,
                Action::Swap(_) => 1,
                Action::SwapClaim(_) => 1,
                Action::UndelegateClaim(_) => 1,
                Action::DelegatorVote(_) => 1,
                _ => 0,
            })
            .sum()
    }

    /// Helper function for decrypting the memo on the transaction given an FVK.
    ///
    /// Will return an Error if there is no memo.
    pub fn decrypt_memo(&self, fvk: &FullViewingKey) -> anyhow::Result<MemoPlaintext> {
        // Error if we don't have an encrypted memo field to decrypt.
        if self.transaction_body().memo.is_none() {
            return Err(anyhow::anyhow!("no memo"));
        }

        // Any output will let us decrypt the memo.
        if let Some(output) = self.outputs().next() {
            // First decrypt the wrapped memo key on the output.
            let ovk_wrapped_key = output.body.ovk_wrapped_key.clone();
            let shared_secret = Note::decrypt_key(
                ovk_wrapped_key,
                output.body.note_payload.note_commitment,
                output.body.balance_commitment,
                fvk.outgoing(),
                &output.body.note_payload.ephemeral_key,
            );

            let wrapped_memo_key = &output.body.wrapped_memo_key;
            let memo_key: PayloadKey = match shared_secret {
                Ok(shared_secret) => {
                    let payload_key =
                        PayloadKey::derive(&shared_secret, &output.body.note_payload.ephemeral_key);
                    wrapped_memo_key.decrypt_outgoing(&payload_key)?
                }
                Err(_) => wrapped_memo_key
                    .decrypt(output.body.note_payload.ephemeral_key, fvk.incoming())?,
            };

            // Now we can use the memo key to decrypt the memo.
            let tx_body = self.transaction_body();
            let memo_ciphertext = tx_body
                .memo
                .as_ref()
                .expect("memo field exists on this transaction");
            let decrypted_memo = MemoCiphertext::decrypt(&memo_key, memo_ciphertext.clone())?;

            // The memo is shared across all outputs, so we can stop here.
            return Ok(decrypted_memo);
        }

        // If we got here, we were unable to decrypt the memo.
        Err(anyhow::anyhow!("unable to decrypt memo"))
    }

    pub fn payload_keys(
        &self,
        fvk: &FullViewingKey,
    ) -> anyhow::Result<BTreeMap<StateCommitment, PayloadKey>> {
        let mut result = BTreeMap::new();

        for action in self.actions() {
            match action {
                Action::Swap(swap) => {
                    let commitment = swap.body.payload.commitment;
                    let payload_key = PayloadKey::derive_swap(fvk.outgoing(), commitment);

                    result.insert(commitment, payload_key);
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
                Action::SwapClaim(_)
                | Action::Spend(_)
                | Action::Delegate(_)
                | Action::Undelegate(_)
                | Action::UndelegateClaim(_)
                | Action::ValidatorDefinition(_)
                | Action::IbcRelay(_)
                | Action::ProposalSubmit(_)
                | Action::ProposalWithdraw(_)
                | Action::ValidatorVote(_)
                | Action::DelegatorVote(_)
                | Action::ProposalDepositClaim(_)
                | Action::PositionOpen(_)
                | Action::PositionClose(_)
                | Action::PositionWithdraw(_)
                | Action::PositionRewardClaim(_)
                | Action::Ics20Withdrawal(_)
                | Action::DaoSpend(_)
                | Action::DaoOutput(_)
                | Action::DaoDeposit(_) => {}
            }
        }

        Ok(result)
    }

    pub fn view_from_perspective(&self, txp: &TransactionPerspective) -> TransactionView {
        let mut action_views = Vec::new();

        let mut memo_plaintext: Option<MemoPlaintext> = None;
        let mut memo_ciphertext: Option<MemoCiphertext> = None;

        for action in self.actions() {
            let action_view = action.view_from_perspective(txp);

            // In the case of Output actions, decrypt the transaction memo if this hasn't already been done.
            if let ActionView::Output(output) = &action_view {
                if memo_plaintext.is_none() {
                    memo_plaintext = match self.transaction_body().memo {
                        Some(ciphertext) => {
                            memo_ciphertext = Some(ciphertext.clone());
                            match output {
                                OutputView::Visible {
                                    output: _,
                                    note: _,
                                    payload_key: decrypted_memo_key,
                                } => MemoCiphertext::decrypt(decrypted_memo_key, ciphertext).ok(),
                                OutputView::Opaque { output: _ } => None,
                            }
                        }
                        None => None,
                    }
                }
            }

            action_views.push(action_view);
        }

        let memo_view = match memo_ciphertext {
            Some(ciphertext) => match memo_plaintext {
                Some(plaintext) => {
                    let plaintext_view: MemoPlaintextView = MemoPlaintextView {
                        return_address: txp.view_address(plaintext.return_address),
                        text: plaintext.text,
                    };
                    Some(MemoView::Visible {
                        plaintext: plaintext_view,
                        ciphertext,
                    })
                }
                None => Some(MemoView::Opaque { ciphertext }),
            },
            None => None,
        };

        let detection_data =
            self.transaction_body()
                .detection_data
                .as_ref()
                .map(|detection_data| DetectionData {
                    fmd_clues: detection_data.fmd_clues.clone(),
                });

        TransactionView {
            body_view: TransactionBodyView {
                action_views,
                transaction_parameters: self.transaction_parameters(),
                fee: self.transaction_body().fee,
                detection_data,
                memo_view,
            },
            binding_sig: self.binding_sig,
            anchor: self.anchor,
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

    pub fn undelegate_claims(&self) -> impl Iterator<Item = &UndelegateClaim> {
        self.actions().filter_map(|action| {
            if let Action::UndelegateClaim(d) = action {
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

    pub fn delegator_votes(&self) -> impl Iterator<Item = &DelegatorVote> {
        self.actions().filter_map(|action| {
            if let Action::DelegatorVote(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn ibc_actions(&self) -> impl Iterator<Item = &IbcRelay> {
        self.actions().filter_map(|action| {
            if let Action::IbcRelay(ibc_action) = action {
                Some(ibc_action)
            } else {
                None
            }
        })
    }

    pub fn validator_definitions(
        &self,
    ) -> impl Iterator<Item = &penumbra_stake::validator::Definition> {
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

    pub fn dao_deposits(&self) -> impl Iterator<Item = &DaoDeposit> {
        self.actions().filter_map(|action| {
            if let Action::DaoDeposit(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn dao_spends(&self) -> impl Iterator<Item = &DaoSpend> {
        self.actions().filter_map(|action| {
            if let Action::DaoSpend(s) = action {
                Some(s)
            } else {
                None
            }
        })
    }

    pub fn spends(&self) -> impl Iterator<Item = &Spend> {
        self.actions().filter_map(|action| {
            if let Action::Spend(s) = action {
                Some(s)
            } else {
                None
            }
        })
    }

    pub fn dao_outputs(&self) -> impl Iterator<Item = &DaoOutput> {
        self.actions().filter_map(|action| {
            if let Action::DaoOutput(o) = action {
                Some(o)
            } else {
                None
            }
        })
    }

    pub fn position_openings(&self) -> impl Iterator<Item = &PositionOpen> {
        self.actions().filter_map(|action| {
            if let Action::PositionOpen(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn position_closings(&self) -> impl Iterator<Item = &PositionClose> {
        self.actions().filter_map(|action| {
            if let Action::PositionClose(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn transaction_body(&self) -> TransactionBody {
        self.transaction_body.clone()
    }

    pub fn transaction_parameters(&self) -> TransactionParameters {
        self.transaction_body.transaction_parameters.clone()
    }

    pub fn binding_sig(&self) -> &Signature<Binding> {
        &self.binding_sig
    }

    pub fn id(&self) -> Id {
        use sha2::{Digest, Sha256};

        let tx_bytes: Vec<u8> = self.clone().try_into().expect("can serialize transaction");
        let mut id_bytes = [0; 32];
        id_bytes[..].copy_from_slice(Sha256::digest(&tx_bytes).as_slice());

        Id(id_bytes)
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

impl DomainType for TransactionParameters {
    type Proto = pbt::TransactionParameters;
}

impl TryFrom<pbt::TransactionParameters> for TransactionParameters {
    type Error = Error;

    fn try_from(proto: pbt::TransactionParameters) -> anyhow::Result<Self, Self::Error> {
        Ok(TransactionParameters {
            expiry_height: proto.expiry_height,
            chain_id: proto.chain_id,
        })
    }
}

impl From<TransactionParameters> for pbt::TransactionParameters {
    fn from(msg: TransactionParameters) -> Self {
        pbt::TransactionParameters {
            expiry_height: msg.expiry_height,
            chain_id: msg.chain_id,
        }
    }
}

impl DomainType for DetectionData {
    type Proto = pbt::DetectionData;
}

impl TryFrom<pbt::DetectionData> for DetectionData {
    type Error = Error;

    fn try_from(proto: pbt::DetectionData) -> anyhow::Result<Self, Self::Error> {
        let fmd_clues = proto
            .fmd_clues
            .into_iter()
            .map(|x| x.try_into())
            .collect::<Result<Vec<Clue>, Error>>()?;
        Ok(DetectionData { fmd_clues })
    }
}

impl From<DetectionData> for pbt::DetectionData {
    fn from(msg: DetectionData) -> Self {
        let fmd_clues = msg.fmd_clues.into_iter().map(|x| x.into()).collect();

        pbt::DetectionData { fmd_clues }
    }
}

impl DomainType for TransactionBody {
    type Proto = pbt::TransactionBody;
}

impl From<TransactionBody> for pbt::TransactionBody {
    fn from(msg: TransactionBody) -> Self {
        let encrypted_memo: pbt::MemoData = match msg.memo {
            Some(memo) => pbt::MemoData {
                encrypted_memo: memo.0.to_vec(),
            },
            None => pbt::MemoData {
                encrypted_memo: Default::default(),
            },
        };

        pbt::TransactionBody {
            actions: msg.actions.into_iter().map(|x| x.into()).collect(),
            transaction_parameters: Some(msg.transaction_parameters.into()),
            fee: Some(msg.fee.into()),
            detection_data: msg.detection_data.map(|x| x.into()),
            memo_data: Some(encrypted_memo),
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
                    .context("action malformed while parsing transaction body")?,
            );
        }

        let fee: Fee = proto
            .fee
            .ok_or_else(|| anyhow::anyhow!("transaction body missing fee"))?
            .try_into()
            .context("fee malformed")?;

        let encrypted_memo = proto
            .memo_data
            .ok_or_else(|| anyhow::anyhow!("transaction body missing memo data field"))?
            .encrypted_memo;

        let memo: Option<MemoCiphertext> = if encrypted_memo.is_empty() {
            None
        } else {
            Some(
                encrypted_memo[..]
                    .try_into()
                    .context("encrypted memo malformed while parsing transaction body")?,
            )
        };

        let detection_data = match proto.detection_data {
            Some(data) => Some(
                data.try_into()
                    .context("detection data malformed while parsing transaction body")?,
            ),
            None => None,
        };

        let transaction_parameters = proto
            .transaction_parameters
            .ok_or_else(|| anyhow::anyhow!("transaction body missing transaction parameters"))?
            .try_into()
            .context("transaction parameters malformed")?;

        Ok(TransactionBody {
            actions,
            transaction_parameters,
            fee,
            detection_data,
            memo,
        })
    }
}

impl DomainType for Transaction {
    type Proto = pbt::Transaction;
}

impl From<Transaction> for pbt::Transaction {
    fn from(msg: Transaction) -> Self {
        let sig_bytes: [u8; 64] = msg.binding_sig.into();
        pbt::Transaction {
            body: Some(msg.transaction_body.into()),
            anchor: Some(msg.anchor.into()),
            binding_sig: sig_bytes.to_vec(),
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
            .ok_or_else(|| anyhow::anyhow!("transaction missing body"))?
            .try_into()
            .context("transaction body malformed")?;

        let sig_bytes: [u8; 64] = proto.binding_sig[..]
            .try_into()
            .context("transaction binding signature malformed")?;

        let anchor = proto
            .anchor
            .ok_or_else(|| anyhow::anyhow!("transaction missing anchor"))?
            .try_into()
            .context("transaction anchor malformed")?;

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
        pbt::Transaction::decode(bytes)?.try_into()
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
