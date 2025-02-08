use anyhow::Context;
use decaf377_rdsa::{Binding, Signature};
use penumbra_sdk_asset::{Balance, Value};
use penumbra_sdk_dex::{swap::SwapView, swap_claim::SwapClaimView};
use penumbra_sdk_keys::AddressView;
use penumbra_sdk_proto::{core::transaction::v1 as pbt, DomainType};
use penumbra_sdk_shielded_pool::{OutputView, SpendView};
use serde::{Deserialize, Serialize};

pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
use penumbra_sdk_tct as tct;
pub use transaction_perspective::TransactionPerspective;

use crate::{
    memo::MemoCiphertext,
    transaction::{TransactionEffect, TransactionSummary},
    Action, DetectionData, Transaction, TransactionBody, TransactionParameters,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::TransactionView", into = "pbt::TransactionView")]
pub struct TransactionView {
    pub body_view: TransactionBodyView,
    pub binding_sig: Signature<Binding>,
    pub anchor: tct::Root,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "pbt::TransactionBodyView",
    into = "pbt::TransactionBodyView"
)]
pub struct TransactionBodyView {
    pub action_views: Vec<ActionView>,
    pub transaction_parameters: TransactionParameters,
    pub detection_data: Option<DetectionData>,
    pub memo_view: Option<MemoView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::MemoView", into = "pbt::MemoView")]
#[allow(clippy::large_enum_variant)]
pub enum MemoView {
    Visible {
        plaintext: MemoPlaintextView,
        ciphertext: MemoCiphertext,
    },
    Opaque {
        ciphertext: MemoCiphertext,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::MemoPlaintextView", into = "pbt::MemoPlaintextView")]
pub struct MemoPlaintextView {
    pub return_address: AddressView,
    pub text: String,
}

impl TransactionView {
    pub fn transaction(&self) -> Transaction {
        let mut actions = Vec::new();

        for action_view in &self.body_view.action_views {
            actions.push(Action::from(action_view.clone()));
        }

        let memo_ciphertext = match &self.body_view.memo_view {
            Some(memo_view) => match memo_view {
                MemoView::Visible {
                    plaintext: _,
                    ciphertext,
                } => Some(ciphertext),
                MemoView::Opaque { ciphertext } => Some(ciphertext),
            },
            None => None,
        };

        let transaction_parameters = self.body_view.transaction_parameters.clone();
        let detection_data = self.body_view.detection_data.clone();

        Transaction {
            transaction_body: TransactionBody {
                actions,
                transaction_parameters,
                detection_data,
                memo: memo_ciphertext.cloned(),
            },
            binding_sig: self.binding_sig,
            anchor: self.anchor,
        }
    }

    pub fn action_views(&self) -> impl Iterator<Item = &ActionView> {
        self.body_view.action_views.iter()
    }

    /// Acts as a higher-order translator that summarizes a TransactionSummary by consolidating
    /// effects for each unique address.
    fn accumulate_effects(summary: TransactionSummary) -> TransactionSummary {
        use std::collections::BTreeMap;
        let mut keyed_effects: BTreeMap<AddressView, Balance> = BTreeMap::new();
        for effect in summary.effects {
            *keyed_effects.entry(effect.address).or_default() += effect.balance;
        }
        TransactionSummary {
            effects: keyed_effects
                .into_iter()
                .map(|(address, balance)| TransactionEffect { address, balance })
                .collect(),
        }
    }

    /// Produces a TransactionSummary, iterating through each visible action and collecting the effects of the transaction.
    pub fn summary(&self) -> TransactionSummary {
        let mut effects = Vec::new();

        for action_view in &self.body_view.action_views {
            match action_view {
                ActionView::Spend(spend_view) => match spend_view {
                    SpendView::Visible { spend: _, note } => {
                        // Provided imbalance (+)
                        let balance = Balance::from(note.value.value());

                        let address = note.address.clone();

                        effects.push(TransactionEffect { address, balance });
                    }
                    SpendView::Opaque { spend: _ } => continue,
                },
                ActionView::Output(output_view) => match output_view {
                    OutputView::Visible {
                        output: _,
                        note,
                        payload_key: _,
                    } => {
                        // Required imbalance (-)
                        let balance = -Balance::from(note.value.value());

                        let address = note.address.clone();

                        effects.push(TransactionEffect { address, balance });
                    }
                    OutputView::Opaque { output: _ } => continue,
                },
                ActionView::Swap(swap_view) => match swap_view {
                    SwapView::Visible {
                        swap: _,
                        swap_plaintext,
                        output_1: _,
                        output_2: _,
                        claim_tx: _,
                        asset_1_metadata: _,
                        asset_2_metadata: _,
                        batch_swap_output_data: _,
                    } => {
                        let address = AddressView::Opaque {
                            address: swap_plaintext.claim_address.clone(),
                        };

                        let value_fee = Value {
                            amount: swap_plaintext.claim_fee.amount(),
                            asset_id: swap_plaintext.claim_fee.asset_id(),
                        };
                        let value_1 = Value {
                            amount: swap_plaintext.delta_1_i,
                            asset_id: swap_plaintext.trading_pair.asset_1(),
                        };
                        let value_2 = Value {
                            amount: swap_plaintext.delta_2_i,
                            asset_id: swap_plaintext.trading_pair.asset_2(),
                        };

                        // Required imbalance (-)
                        let mut balance = Balance::default();
                        balance -= value_1;
                        balance -= value_2;
                        balance -= value_fee;

                        effects.push(TransactionEffect { address, balance });
                    }
                    SwapView::Opaque {
                        swap: _,
                        batch_swap_output_data: _,
                        output_1: _,
                        output_2: _,
                        asset_1_metadata: _,
                        asset_2_metadata: _,
                    } => continue,
                },
                ActionView::SwapClaim(swap_claim_view) => match swap_claim_view {
                    SwapClaimView::Visible {
                        swap_claim,
                        output_1,
                        output_2: _,
                        swap_tx: _,
                    } => {
                        let address = AddressView::Opaque {
                            address: output_1.address(),
                        };

                        let value_fee = Value {
                            amount: swap_claim.body.fee.amount(),
                            asset_id: swap_claim.body.fee.asset_id(),
                        };

                        // Provided imbalance (+)
                        let mut balance = Balance::default();
                        balance += value_fee;

                        effects.push(TransactionEffect { address, balance });
                    }
                    SwapClaimView::Opaque { swap_claim: _ } => continue,
                },
                _ => {} // Fill in other action views as necessary
            }
        }

        let summary = TransactionSummary { effects };

        Self::accumulate_effects(summary)
    }
}

impl DomainType for TransactionView {
    type Proto = pbt::TransactionView;
}

impl TryFrom<pbt::TransactionView> for TransactionView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::TransactionView) -> Result<Self, Self::Error> {
        let binding_sig = v
            .binding_sig
            .ok_or_else(|| anyhow::anyhow!("transaction view missing binding signature"))?
            .try_into()
            .context("transaction binding signature malformed")?;

        let anchor = v
            .anchor
            .ok_or_else(|| anyhow::anyhow!("transaction view missing anchor"))?
            .try_into()
            .context("transaction anchor malformed")?;

        let body_view = v
            .body_view
            .ok_or_else(|| anyhow::anyhow!("transaction view missing body"))?
            .try_into()
            .context("transaction body malformed")?;

        Ok(Self {
            body_view,
            binding_sig,
            anchor,
        })
    }
}

impl TryFrom<pbt::TransactionBodyView> for TransactionBodyView {
    type Error = anyhow::Error;

    fn try_from(body_view: pbt::TransactionBodyView) -> Result<Self, Self::Error> {
        let mut action_views = Vec::<ActionView>::new();
        for av in body_view.action_views.clone() {
            action_views.push(av.try_into()?);
        }

        let memo_view: Option<MemoView> = match body_view.memo_view {
            Some(mv) => match mv.memo_view {
                Some(x) => match x {
                    pbt::memo_view::MemoView::Visible(v) => Some(MemoView::Visible {
                        plaintext: v
                            .plaintext
                            .ok_or_else(|| {
                                anyhow::anyhow!("transaction view memo missing memo plaintext")
                            })?
                            .try_into()?,
                        ciphertext: v
                            .ciphertext
                            .ok_or_else(|| {
                                anyhow::anyhow!("transaction view memo missing memo ciphertext")
                            })?
                            .try_into()?,
                    }),
                    pbt::memo_view::MemoView::Opaque(v) => Some(MemoView::Opaque {
                        ciphertext: v
                            .ciphertext
                            .ok_or_else(|| {
                                anyhow::anyhow!("transaction view memo missing memo ciphertext")
                            })?
                            .try_into()?,
                    }),
                },
                None => None,
            },
            None => None,
        };

        let transaction_parameters = body_view
            .transaction_parameters
            .ok_or_else(|| anyhow::anyhow!("transaction view missing transaction parameters view"))?
            .try_into()?;

        // Iterate through the detection_data vec, and convert each FMD clue.
        let fmd_clues = body_view
            .detection_data
            .map(|dd| {
                dd.fmd_clues
                    .into_iter()
                    .map(|fmd| fmd.try_into())
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;

        let detection_data = fmd_clues.map(|fmd_clues| DetectionData { fmd_clues });

        Ok(TransactionBodyView {
            action_views,
            transaction_parameters,
            detection_data,
            memo_view,
        })
    }
}

impl From<TransactionView> for pbt::TransactionView {
    fn from(v: TransactionView) -> Self {
        Self {
            body_view: Some(v.body_view.into()),
            anchor: Some(v.anchor.into()),
            binding_sig: Some(v.binding_sig.into()),
        }
    }
}

impl From<TransactionBodyView> for pbt::TransactionBodyView {
    fn from(v: TransactionBodyView) -> Self {
        Self {
            action_views: v.action_views.into_iter().map(Into::into).collect(),
            transaction_parameters: Some(v.transaction_parameters.into()),
            detection_data: v.detection_data.map(Into::into),
            memo_view: v.memo_view.map(|m| m.into()),
        }
    }
}

impl From<MemoView> for pbt::MemoView {
    fn from(v: MemoView) -> Self {
        Self {
            memo_view: match v {
                MemoView::Visible {
                    plaintext,
                    ciphertext,
                } => Some(pbt::memo_view::MemoView::Visible(pbt::memo_view::Visible {
                    plaintext: Some(plaintext.into()),
                    ciphertext: Some(ciphertext.into()),
                })),
                MemoView::Opaque { ciphertext } => {
                    Some(pbt::memo_view::MemoView::Opaque(pbt::memo_view::Opaque {
                        ciphertext: Some(ciphertext.into()),
                    }))
                }
            },
        }
    }
}

impl TryFrom<pbt::MemoView> for MemoView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::MemoView) -> Result<Self, Self::Error> {
        match v
            .memo_view
            .ok_or_else(|| anyhow::anyhow!("missing memo field"))?
        {
            pbt::memo_view::MemoView::Visible(x) => Ok(MemoView::Visible {
                plaintext: x
                    .plaintext
                    .ok_or_else(|| anyhow::anyhow!("missing plaintext field"))?
                    .try_into()?,
                ciphertext: x
                    .ciphertext
                    .ok_or_else(|| anyhow::anyhow!("missing ciphertext field"))?
                    .try_into()?,
            }),
            pbt::memo_view::MemoView::Opaque(x) => Ok(MemoView::Opaque {
                ciphertext: x
                    .ciphertext
                    .ok_or_else(|| anyhow::anyhow!("missing ciphertext field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<MemoPlaintextView> for pbt::MemoPlaintextView {
    fn from(v: MemoPlaintextView) -> Self {
        Self {
            return_address: Some(v.return_address.into()),
            text: v.text,
        }
    }
}

impl TryFrom<pbt::MemoPlaintextView> for MemoPlaintextView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::MemoPlaintextView) -> Result<Self, Self::Error> {
        let sender: AddressView = v
            .return_address
            .ok_or_else(|| anyhow::anyhow!("memo plan missing memo plaintext"))?
            .try_into()
            .context("return address malformed")?;

        let text: String = v.text;

        Ok(Self {
            return_address: sender,
            text,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use decaf377::Fr;
    use decaf377::{Element, Fq};
    use decaf377_rdsa::{Domain, VerificationKey};
    use penumbra_sdk_asset::{
        asset::{self, Cache, Id},
        balance::Commitment,
        STAKING_TOKEN_ASSET_ID,
    };
    use penumbra_sdk_dex::swap::proof::SwapProof;
    use penumbra_sdk_dex::swap::{SwapCiphertext, SwapPayload};
    use penumbra_sdk_dex::Swap;
    use penumbra_sdk_dex::{
        swap::{SwapPlaintext, SwapPlan},
        TradingPair,
    };
    use penumbra_sdk_fee::Fee;
    use penumbra_sdk_keys::keys::Bip44Path;
    use penumbra_sdk_keys::keys::{SeedPhrase, SpendKey};
    use penumbra_sdk_keys::{
        symmetric::{OvkWrappedKey, WrappedMemoKey},
        test_keys, Address, FullViewingKey, PayloadKey,
    };
    use penumbra_sdk_num::Amount;
    use penumbra_sdk_proof_params::GROTH16_PROOF_LENGTH_BYTES;
    use penumbra_sdk_sct::Nullifier;
    use penumbra_sdk_shielded_pool::Rseed;
    use penumbra_sdk_shielded_pool::{output, spend, Note, NoteView, OutputPlan, SpendPlan};
    use penumbra_sdk_tct::structure::Hash;
    use penumbra_sdk_tct::StateCommitment;
    use rand_core::OsRng;
    use std::ops::Deref;

    use crate::{
        plan::{CluePlan, DetectionDataPlan},
        view, ActionPlan, TransactionPlan,
    };

    #[cfg(test)]
    fn dummy_sig<D: Domain>() -> Signature<D> {
        Signature::from([0u8; 64])
    }

    #[cfg(test)]
    fn dummy_pk<D: Domain>() -> VerificationKey<D> {
        VerificationKey::try_from(Element::default().vartime_compress().0)
            .expect("creating a dummy verification key should work")
    }

    #[cfg(test)]
    fn dummy_commitment() -> Commitment {
        Commitment(Element::default())
    }

    #[cfg(test)]
    fn dummy_proof_spend() -> spend::SpendProof {
        spend::SpendProof::try_from(
            penumbra_sdk_proto::penumbra::core::component::shielded_pool::v1::ZkSpendProof {
                inner: vec![0u8; GROTH16_PROOF_LENGTH_BYTES],
            },
        )
        .expect("creating a dummy proof should work")
    }

    #[cfg(test)]
    fn dummy_proof_output() -> output::OutputProof {
        output::OutputProof::try_from(
            penumbra_sdk_proto::penumbra::core::component::shielded_pool::v1::ZkOutputProof {
                inner: vec![0u8; GROTH16_PROOF_LENGTH_BYTES],
            },
        )
        .expect("creating a dummy proof should work")
    }

    #[cfg(test)]
    fn dummy_proof_swap() -> SwapProof {
        SwapProof::try_from(
            penumbra_sdk_proto::penumbra::core::component::dex::v1::ZkSwapProof {
                inner: vec![0u8; GROTH16_PROOF_LENGTH_BYTES],
            },
        )
        .expect("creating a dummy proof should work")
    }

    #[cfg(test)]
    fn dummy_spend() -> spend::Spend {
        use penumbra_sdk_shielded_pool::EncryptedBackref;

        spend::Spend {
            body: spend::Body {
                balance_commitment: dummy_commitment(),
                nullifier: Nullifier(Fq::default()),
                rk: dummy_pk(),
                encrypted_backref: EncryptedBackref::dummy(),
            },
            auth_sig: dummy_sig(),
            proof: dummy_proof_spend(),
        }
    }

    #[cfg(test)]
    fn dummy_output() -> output::Output {
        output::Output {
            body: output::Body {
                note_payload: penumbra_sdk_shielded_pool::NotePayload {
                    note_commitment: penumbra_sdk_shielded_pool::note::StateCommitment(
                        Fq::default(),
                    ),
                    ephemeral_key: [0u8; 32]
                        .as_slice()
                        .try_into()
                        .expect("can create dummy ephemeral key"),
                    encrypted_note: penumbra_sdk_shielded_pool::NoteCiphertext([0u8; 176]),
                },
                balance_commitment: dummy_commitment(),
                ovk_wrapped_key: OvkWrappedKey([0u8; 48]),
                wrapped_memo_key: WrappedMemoKey([0u8; 48]),
            },
            proof: dummy_proof_output(),
        }
    }

    #[cfg(test)]
    fn dummy_swap_plaintext() -> SwapPlaintext {
        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1 = Amount::from(1u64);
        let delta_2 = Amount::from(0u64);
        let fee = Fee::default();

        let swap_plaintext = SwapPlaintext::new(
            &mut OsRng,
            trading_pair,
            delta_1,
            delta_2,
            fee,
            claim_address,
        );

        swap_plaintext
    }

    #[cfg(test)]
    fn dummy_swap() -> Swap {
        use penumbra_sdk_dex::swap::Body;

        let seed_phrase = SeedPhrase::generate(OsRng);
        let sk_recipient = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk_recipient = sk_recipient.full_viewing_key();
        let ivk_recipient = fvk_recipient.incoming();
        let (claim_address, _dtk_d) = ivk_recipient.payment_address(0u32.into());

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1 = Amount::from(1u64);
        let delta_2 = Amount::from(0u64);
        let fee = Fee::default();

        let swap_plaintext = SwapPlaintext::new(
            &mut OsRng,
            trading_pair,
            delta_1,
            delta_2,
            fee,
            claim_address,
        );

        let fee_blinding = Fr::from(0u64);
        let fee_commitment = swap_plaintext.claim_fee.commit(fee_blinding);

        let swap_payload = SwapPayload {
            encrypted_swap: SwapCiphertext([0u8; 272]),
            commitment: StateCommitment::try_from([0; 32]).expect("state commitment"),
        };

        Swap {
            body: Body {
                trading_pair: trading_pair,
                delta_1_i: delta_1,
                delta_2_i: delta_2,
                fee_commitment: fee_commitment,
                payload: swap_payload,
            },
            proof: dummy_proof_swap(),
        }
    }

    #[cfg(test)]
    fn dummy_note_view(
        address: Address,
        value: Value,
        cache: &Cache,
        fvk: &FullViewingKey,
    ) -> NoteView {
        let note = Note::from_parts(address, value, Rseed::generate(&mut OsRng))
            .expect("generate dummy note");

        NoteView {
            value: note.value().view_with_cache(cache),
            rseed: note.rseed(),
            address: fvk.view_address(note.address()),
        }
    }

    #[cfg(test)]
    fn convert_note(cache: &Cache, fvk: &FullViewingKey, note: &Note) -> NoteView {
        NoteView {
            value: note.value().view_with_cache(cache),
            rseed: note.rseed(),
            address: fvk.view_address(note.address()),
        }
    }

    #[cfg(test)]
    fn convert_action(
        cache: &Cache,
        fvk: &FullViewingKey,
        action: &ActionPlan,
    ) -> Option<ActionView> {
        use view::action_view::SpendView;

        match action {
            ActionPlan::Output(x) => Some(ActionView::Output(
                penumbra_sdk_shielded_pool::OutputView::Visible {
                    output: dummy_output(),
                    note: convert_note(cache, fvk, &x.output_note()),
                    payload_key: PayloadKey::from([0u8; 32]),
                },
            )),
            ActionPlan::Spend(x) => Some(ActionView::Spend(SpendView::Visible {
                spend: dummy_spend(),
                note: convert_note(cache, fvk, &x.note),
            })),
            ActionPlan::ValidatorDefinition(_) => None,
            ActionPlan::Swap(x) => Some(ActionView::Swap(SwapView::Visible {
                swap: dummy_swap(),
                swap_plaintext: dummy_swap_plaintext(),
                output_1: Some(dummy_note_view(
                    x.swap_plaintext.claim_address.clone(),
                    x.swap_plaintext.claim_fee.0,
                    cache,
                    fvk,
                )),
                output_2: None,
                claim_tx: None,
                asset_1_metadata: None,
                asset_2_metadata: None,
                batch_swap_output_data: None,
            })),
            ActionPlan::SwapClaim(_) => None,
            ActionPlan::ProposalSubmit(_) => None,
            ActionPlan::ProposalWithdraw(_) => None,
            ActionPlan::DelegatorVote(_) => None,
            ActionPlan::ValidatorVote(_) => None,
            ActionPlan::ProposalDepositClaim(_) => None,
            ActionPlan::PositionOpen(_) => None,
            ActionPlan::PositionClose(_) => None,
            ActionPlan::PositionWithdraw(_) => None,
            ActionPlan::Delegate(_) => None,
            ActionPlan::Undelegate(_) => None,
            ActionPlan::UndelegateClaim(_) => None,
            ActionPlan::Ics20Withdrawal(_) => None,
            ActionPlan::CommunityPoolSpend(_) => None,
            ActionPlan::CommunityPoolOutput(_) => None,
            ActionPlan::CommunityPoolDeposit(_) => None,
            ActionPlan::ActionDutchAuctionSchedule(_) => None,
            ActionPlan::ActionDutchAuctionEnd(_) => None,
            ActionPlan::ActionDutchAuctionWithdraw(_) => None,
            ActionPlan::IbcAction(_) => todo!(),
            ActionPlan::ActionLiquidityTournamentVote(_) => todo!(),
        }
    }

    #[test]
    fn test_internal_transfer_transaction_summary() {
        // Generate two notes controlled by the test address.
        let value = Value {
            amount: 100u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);

        let value2 = Value {
            amount: 50u64.into(),
            asset_id: Id(Fq::rand(&mut OsRng)),
        };
        let note2 = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value2);

        let value3 = Value {
            amount: 75u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        for _ in 0..5 {
            let random_note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);
            sct.insert(tct::Witness::Keep, random_note.commit())
                .unwrap();
        }
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        sct.insert(tct::Witness::Keep, note2.commit()).unwrap();

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
                OutputPlan::new(&mut OsRng, value3, test_keys::ADDRESS_1.deref().clone()).into(),
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

        let transaction_view = TransactionView {
            anchor: penumbra_sdk_tct::Root(Hash::zero()),
            binding_sig: Signature::from([0u8; 64]),
            body_view: TransactionBodyView {
                action_views: plan
                    .actions
                    .iter()
                    .filter_map(|x| {
                        convert_action(&Cache::with_known_assets(), &test_keys::FULL_VIEWING_KEY, x)
                    })
                    .collect(),
                transaction_parameters: plan.transaction_parameters.clone(),
                detection_data: None,
                memo_view: None,
            },
        };

        let transaction_summary = TransactionView::summary(&transaction_view);

        assert_eq!(transaction_summary.effects.len(), 2);
    }

    #[test]
    fn test_swap_transaction_summary() {
        // Generate two notes controlled by the test address.
        let value = Value {
            amount: 100u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };
        let note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);

        let value2 = Value {
            amount: 50u64.into(),
            asset_id: Id(Fq::rand(&mut OsRng)),
        };
        let note2 = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value2);

        let value3 = Value {
            amount: 75u64.into(),
            asset_id: *STAKING_TOKEN_ASSET_ID,
        };

        // Record that note in an SCT, where we can generate an auth path.
        let mut sct = tct::Tree::new();
        for _ in 0..5 {
            let random_note = Note::generate(&mut OsRng, &test_keys::ADDRESS_0, value);
            sct.insert(tct::Witness::Keep, random_note.commit())
                .unwrap();
        }
        sct.insert(tct::Witness::Keep, note.commit()).unwrap();
        sct.insert(tct::Witness::Keep, note2.commit()).unwrap();

        let auth_path = sct.witness(note.commit()).unwrap();
        let auth_path2 = sct.witness(note2.commit()).unwrap();

        let gm = asset::Cache::with_known_assets().get_unit("gm").unwrap();
        let gn = asset::Cache::with_known_assets().get_unit("gn").unwrap();
        let trading_pair = TradingPair::new(gm.id(), gn.id());

        let delta_1 = Amount::from(100_000u64);
        let delta_2 = Amount::from(0u64);
        let fee = Fee::default();
        let claim_address: Address = test_keys::ADDRESS_0.deref().clone();
        let plaintext = SwapPlaintext::new(
            &mut OsRng,
            trading_pair,
            delta_1,
            delta_2,
            fee,
            claim_address,
        );

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
                OutputPlan::new(&mut OsRng, value3, test_keys::ADDRESS_1.deref().clone()).into(),
                SwapPlan::new(&mut OsRng, plaintext.clone()).into(),
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

        let transaction_view = TransactionView {
            anchor: penumbra_sdk_tct::Root(Hash::zero()),
            binding_sig: Signature::from([0u8; 64]),
            body_view: TransactionBodyView {
                action_views: plan
                    .actions
                    .iter()
                    .filter_map(|x| {
                        convert_action(&Cache::with_known_assets(), &test_keys::FULL_VIEWING_KEY, x)
                    })
                    .collect(),
                transaction_parameters: plan.transaction_parameters.clone(),
                detection_data: None,
                memo_view: None,
            },
        };

        let transaction_summary = TransactionView::summary(&transaction_view);

        assert_eq!(transaction_summary.effects.len(), 3);
    }
}
