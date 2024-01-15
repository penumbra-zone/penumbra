//! Declarative transaction plans, used for transaction authorization and
//! creation.

use anyhow::Result;
use penumbra_community_pool::{CommunityPoolDeposit, CommunityPoolOutput, CommunityPoolSpend};
use penumbra_dex::{
    lp::action::{PositionClose, PositionOpen},
    lp::plan::PositionWithdrawPlan,
    swap::SwapPlan,
    swap_claim::SwapClaimPlan,
};
use penumbra_governance::{
    DelegatorVotePlan, ProposalDepositClaim, ProposalSubmit, ProposalWithdraw, ValidatorVote,
};
use penumbra_ibc::IbcRelay;
use penumbra_keys::{Address, FullViewingKey, PayloadKey};
use penumbra_proto::{core::transaction::v1alpha1 as pb, DomainType};
use penumbra_shielded_pool::{Ics20Withdrawal, OutputPlan, SpendPlan};
use penumbra_stake::{Delegate, Undelegate, UndelegateClaimPlan};
use penumbra_txhash::{EffectHash, EffectingData};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};

mod action;
mod auth;
mod build;
mod clue;
mod detection_data;
mod memo;

pub use action::ActionPlan;
pub use clue::CluePlan;
pub use detection_data::DetectionDataPlan;
pub use memo::MemoPlan;

use crate::TransactionParameters;

/// A declaration of a planned [`Transaction`](crate::Transaction),
/// for use in transaction authorization and creation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(try_from = "pb::TransactionPlan", into = "pb::TransactionPlan")]
pub struct TransactionPlan {
    pub actions: Vec<ActionPlan>,
    pub transaction_parameters: TransactionParameters,
    pub detection_data: Option<DetectionDataPlan>,
    pub memo: Option<MemoPlan>,
}

impl TransactionPlan {
    /// Computes the [`EffectHash`] for the [`Transaction`] described by this
    /// [`TransactionPlan`].
    ///
    /// This method does not require constructing the entire [`Transaction`],
    /// but it does require the associated [`FullViewingKey`] to derive
    /// effecting data that will be fed into the [`EffectHash`].
    ///
    /// This method is not an [`EffectingData`] impl because it needs an extra input,
    /// the FVK, to partially construct the transaction.
    pub fn effect_hash(&self, fvk: &FullViewingKey) -> Result<EffectHash> {
        // This implementation is identical to the one for Transaction, except that we
        // don't need to actually construct the entire `TransactionBody` with
        // complete `Action`s, we just need to construct the bodies of the
        // actions the transaction will have when constructed.

        let mut state = blake2b_simd::Params::new()
            .personal(b"PenumbraEfHs")
            .to_state();

        let parameters_hash = self.transaction_parameters.effect_hash();

        let memo_hash = match self.memo {
            Some(ref memo) => memo.memo()?.effect_hash(),
            None => EffectHash::default(),
        };

        let detection_data_hash = self
            .detection_data
            .as_ref()
            .map(|plan| plan.detection_data().effect_hash())
            // If the detection data is not present, use the all-zero hash to
            // record its absence in the overall effect hash.
            .unwrap_or_default();

        // Hash the fixed data of the transaction body.
        state.update(parameters_hash.as_bytes());
        state.update(memo_hash.as_bytes());
        state.update(detection_data_hash.as_bytes());

        // Hash the number of actions, then each action.
        let num_actions = self.actions.len() as u32;
        state.update(&num_actions.to_le_bytes());

        // If the memo_key is None, then there is no memo, so there will be no
        // outputs that the memo key is passed to, so we can fill in a dummy key.
        let memo_key = self.memo_key().unwrap_or([0u8; 32].into());

        // Hash the effecting data of each action, in the order it appears in the plan,
        // which will be the order it appears in the transaction.
        for action_plan in &self.actions {
            state.update(action_plan.effect_hash(fvk, &memo_key).as_bytes());
        }

        Ok(EffectHash(state.finalize().as_array().clone()))
    }

    pub fn spend_plans(&self) -> impl Iterator<Item = &SpendPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Spend(s) = action {
                Some(s)
            } else {
                None
            }
        })
    }

    pub fn output_plans(&self) -> impl Iterator<Item = &OutputPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Output(o) = action {
                Some(o)
            } else {
                None
            }
        })
    }

    pub fn delegations(&self) -> impl Iterator<Item = &Delegate> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Delegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn undelegations(&self) -> impl Iterator<Item = &Undelegate> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Undelegate(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn undelegate_claim_plans(&self) -> impl Iterator<Item = &UndelegateClaimPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::UndelegateClaim(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn ibc_actions(&self) -> impl Iterator<Item = &IbcRelay> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::IbcAction(ibc_action) = action {
                Some(ibc_action)
            } else {
                None
            }
        })
    }

    pub fn validator_definitions(
        &self,
    ) -> impl Iterator<Item = &penumbra_stake::validator::Definition> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ValidatorDefinition(d) = action {
                Some(d)
            } else {
                None
            }
        })
    }

    pub fn proposal_submits(&self) -> impl Iterator<Item = &ProposalSubmit> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ProposalSubmit(p) = action {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn proposal_withdraws(&self) -> impl Iterator<Item = &ProposalWithdraw> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ProposalWithdraw(p) = action {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn delegator_vote_plans(&self) -> impl Iterator<Item = &DelegatorVotePlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::DelegatorVote(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn validator_votes(&self) -> impl Iterator<Item = &ValidatorVote> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ValidatorVote(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn proposal_deposit_claims(&self) -> impl Iterator<Item = &ProposalDepositClaim> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::ProposalDepositClaim(p) = action {
                Some(p)
            } else {
                None
            }
        })
    }

    pub fn swap_plans(&self) -> impl Iterator<Item = &SwapPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Swap(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn swap_claim_plans(&self) -> impl Iterator<Item = &SwapClaimPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::SwapClaim(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn community_pool_spends(&self) -> impl Iterator<Item = &CommunityPoolSpend> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::CommunityPoolSpend(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn community_pool_deposits(&self) -> impl Iterator<Item = &CommunityPoolDeposit> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::CommunityPoolDeposit(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn community_pool_outputs(&self) -> impl Iterator<Item = &CommunityPoolOutput> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::CommunityPoolOutput(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn position_openings(&self) -> impl Iterator<Item = &PositionOpen> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::PositionOpen(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn position_closings(&self) -> impl Iterator<Item = &PositionClose> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::PositionClose(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn position_withdrawals(&self) -> impl Iterator<Item = &PositionWithdrawPlan> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::PositionWithdraw(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    pub fn ics20_withdrawals(&self) -> impl Iterator<Item = &Ics20Withdrawal> {
        self.actions.iter().filter_map(|action| {
            if let ActionPlan::Withdrawal(v) = action {
                Some(v)
            } else {
                None
            }
        })
    }

    /// Convenience method to get all the destination addresses for each `OutputPlan`s.
    pub fn dest_addresses(&self) -> Vec<Address> {
        self.output_plans().map(|plan| plan.dest_address).collect()
    }

    /// Convenience method to get the number of `OutputPlan`s in this transaction.
    pub fn num_outputs(&self) -> usize {
        self.output_plans().count()
    }

    /// Method to populate the detection data for this transaction plan.
    pub fn populate_detection_data<R: CryptoRng + Rng>(
        &mut self,
        mut rng: R,
        precision_bits: usize,
    ) {
        // Add one clue per recipient.
        let mut clue_plans = vec![];
        for dest_address in self.dest_addresses() {
            clue_plans.push(CluePlan::new(&mut rng, dest_address, precision_bits));
        }

        // Now add dummy clues until we have one clue per output.
        let num_dummy_clues = self.num_outputs() - clue_plans.len();
        for _ in 0..num_dummy_clues {
            let dummy_address = Address::dummy(&mut rng);
            clue_plans.push(CluePlan::new(&mut rng, dummy_address, precision_bits));
        }

        if !clue_plans.is_empty() {
            self.detection_data = Some(DetectionDataPlan { clue_plans });
        } else {
            self.detection_data = None;
        }
    }

    /// Convenience method to grab the `MemoKey` from the plan.
    pub fn memo_key(&self) -> Option<PayloadKey> {
        self.memo.as_ref().map(|memo_plan| memo_plan.key.clone())
    }
}

impl DomainType for TransactionPlan {
    type Proto = pb::TransactionPlan;
}

impl From<TransactionPlan> for pb::TransactionPlan {
    fn from(msg: TransactionPlan) -> Self {
        Self {
            actions: msg.actions.into_iter().map(Into::into).collect(),
            transaction_parameters: Some(msg.transaction_parameters.into()),
            detection_data: msg.detection_data.map(Into::into),
            memo: msg.memo.map(Into::into),
        }
    }
}

impl TryFrom<pb::TransactionPlan> for TransactionPlan {
    type Error = anyhow::Error;
    fn try_from(value: pb::TransactionPlan) -> Result<Self, Self::Error> {
        Ok(Self {
            actions: value
                .actions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            transaction_parameters: value
                .transaction_parameters
                .ok_or_else(|| anyhow::anyhow!("transaction plan missing transaction parameters"))?
                .try_into()?,
            detection_data: value.detection_data.map(TryInto::try_into).transpose()?,
            memo: value.memo.map(TryInto::try_into).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use penumbra_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
    use penumbra_dex::{swap::SwapPlaintext, swap::SwapPlan, TradingPair};
    use penumbra_fee::Fee;
    use penumbra_keys::{
        keys::{Bip44Path, SeedPhrase, SpendKey},
        Address,
    };
    use penumbra_shielded_pool::Note;
    use penumbra_shielded_pool::{OutputPlan, SpendPlan};
    use penumbra_tct as tct;
    use penumbra_txhash::EffectingData as _;
    use rand_core::OsRng;

    use crate::{
        memo::MemoPlaintext,
        plan::{CluePlan, DetectionDataPlan, MemoPlan, TransactionPlan},
        TransactionParameters, WitnessData,
    };

    /// This isn't an exhaustive test, but we don't currently have a
    /// great way to generate actions for randomized testing.
    ///
    /// All we hope to check here is that, for a basic transaction plan,
    /// we compute the same auth hash for the plan and for the transaction.
    #[test]
    fn plan_effect_hash_matches_transaction_effect_hash() {
        let rng = OsRng;
        let seed_phrase = SeedPhrase::generate(rng);
        let sk = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
        let fvk = sk.full_viewing_key();
        let (addr, _dtk) = fvk.incoming().payment_address(0u32.into());

        let mut sct = tct::Tree::new();

        let note0 = Note::generate(
            &mut OsRng,
            &addr,
            Value {
                amount: 10000u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );
        let note1 = Note::generate(
            &mut OsRng,
            &addr,
            Value {
                amount: 20000u64.into(),
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
        );

        sct.insert(tct::Witness::Keep, note0.commit()).unwrap();
        sct.insert(tct::Witness::Keep, note1.commit()).unwrap();

        let trading_pair = TradingPair::new(
            asset::Cache::with_known_assets()
                .get_unit("nala")
                .unwrap()
                .id(),
            asset::Cache::with_known_assets()
                .get_unit("upenumbra")
                .unwrap()
                .id(),
        );

        let swap_plaintext = SwapPlaintext::new(
            &mut OsRng,
            trading_pair,
            100000u64.into(),
            1u64.into(),
            Fee(Value {
                amount: 3u64.into(),
                asset_id: asset::Cache::with_known_assets()
                    .get_unit("upenumbra")
                    .unwrap()
                    .id(),
            }),
            addr,
        );

        let mut rng = OsRng;

        let memo_plaintext = MemoPlaintext::new(Address::dummy(&mut rng), "".to_string()).unwrap();
        let plan = TransactionPlan {
            // Put outputs first to check that the auth hash
            // computation is not affected by plan ordering.
            actions: vec![
                OutputPlan::new(
                    &mut OsRng,
                    Value {
                        amount: 30000u64.into(),
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    },
                    addr.clone(),
                )
                .into(),
                SpendPlan::new(&mut OsRng, note0, 0u64.into()).into(),
                SpendPlan::new(&mut OsRng, note1, 1u64.into()).into(),
                SwapPlan::new(&mut OsRng, swap_plaintext).into(),
            ],
            transaction_parameters: TransactionParameters {
                expiry_height: 0,
                fee: Fee::default(),
                chain_id: "penumbra-test".to_string(),
            },
            detection_data: Some(DetectionDataPlan {
                clue_plans: vec![CluePlan::new(&mut OsRng, addr, 1)],
            }),
            memo: Some(MemoPlan::new(&mut OsRng, memo_plaintext.clone()).unwrap()),
        };

        println!("{}", serde_json::to_string_pretty(&plan).unwrap());

        let plan_effect_hash = plan.effect_hash(fvk).unwrap();

        let auth_data = plan.authorize(rng, &sk).unwrap();
        let witness_data = WitnessData {
            anchor: sct.root(),
            state_commitment_proofs: plan
                .spend_plans()
                .map(|spend: &SpendPlan| {
                    (
                        spend.note.commit(),
                        sct.witness(spend.note.commit()).unwrap(),
                    )
                })
                .collect(),
        };
        let transaction = plan.build(fvk, &witness_data, &auth_data).unwrap();

        let transaction_effect_hash = transaction.effect_hash();

        assert_eq!(plan_effect_hash, transaction_effect_hash);

        let decrypted_memo = transaction.decrypt_memo(fvk).expect("can decrypt memo");
        assert_eq!(decrypted_memo, memo_plaintext);

        // TODO: fix this and move into its own test?
        // // Also check the concurrent build results in the same effect hash.
        // let rt = Runtime::new().unwrap();
        // let transaction = rt
        //     .block_on(async move {
        //         plan.build_concurrent(&mut OsRng, fvk, auth_data, witness_data)
        //             .await
        //     })
        //     .expect("can build");
        // assert_eq!(plan_effect_hash, transaction.effect_hash());
    }
}
