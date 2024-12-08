use anyhow::Context;
use decaf377_rdsa::{Binding, Signature};
use penumbra_asset::{Balance, Value};
use penumbra_dex::{swap::SwapView, swap_claim::SwapClaimView};
use penumbra_keys::AddressView;
use penumbra_proto::{core::transaction::v1 as pbt, DomainType};

use penumbra_shielded_pool::{OutputView, SpendView};
use serde::{Deserialize, Serialize};

pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
use penumbra_tct as tct;
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

    pub fn summary(&self) -> TransactionSummary {
        let mut effects = Vec::new();

        for action_view in &self.body_view.action_views {
            match action_view {
                ActionView::Spend(spend_view) => match spend_view {
                    SpendView::Visible { spend: _, note } => {
                        // Provided imbalance (+)
                        let balance = Balance::from(note.value.value());

                        let address = AddressView::Opaque {
                            address: note.address(),
                        };

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

                        let address = AddressView::Opaque {
                            address: note.address(),
                        };

                        effects.push(TransactionEffect { address, balance });
                    }
                    OutputView::Opaque { output: _ } => continue,
                },
                ActionView::Swap(swap_view) => match swap_view {
                    SwapView::Visible {
                        swap: _,
                        swap_plaintext,
                        output_1,
                        output_2: _,
                        claim_tx: _,
                        asset_1_metadata: _,
                        asset_2_metadata: _,
                        batch_swap_output_data: _,
                    } => {
                        let address = AddressView::Opaque {
                            address: output_1.clone().expect("sender address").address(),
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
                _ => {} // Fill in other action views as neccessary
            }
        }

        TransactionSummary { effects }
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
