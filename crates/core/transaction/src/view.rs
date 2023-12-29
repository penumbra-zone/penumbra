use anyhow::Context;
use decaf377_rdsa::{Binding, Signature};
use penumbra_keys::AddressView;
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};

use serde::{Deserialize, Serialize};

pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
use penumbra_tct as tct;
pub use transaction_perspective::TransactionPerspective;

use crate::{
    memo::MemoCiphertext, Action, DetectionData, Transaction, TransactionBody,
    TransactionParameters,
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
