use decaf377_fmd::Clue;
use penumbra_crypto::{memo::MemoPlaintext, transaction::Fee, AddressView};
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};
use serde::{Deserialize, Serialize};

pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
use decaf377_rdsa::{Binding, Signature};
use penumbra_tct as tct;
pub use transaction_perspective::TransactionPerspective;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::TransactionView", into = "pbt::TransactionView")]
pub struct TransactionView {
    pub transaction_body_view: TransactionBodyView,
    pub binding_sig: Signature<Binding>,
    pub anchor: tct::Root,
}
#[derive(Clone, Debug, Default)]
pub struct TransactionBodyView {
    pub action_views: Vec<ActionView>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
    pub fmd_clues: Vec<Clue>,
    pub memo: Option<MemoPlaintext>,
    pub address_views: Vec<AddressView>,
}

impl DomainType for TransactionView {
    type Proto = pbt::TransactionView;
}

impl TryFrom<pbt::TransactionView> for TransactionView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::TransactionView) -> Result<Self, Self::Error> {
        Ok(Self {
            transaction_body_view: TransactionBodyView {
                action_views: v
                    .body_view
                    .unwrap()
                    .action_views
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                expiry_height: v.body_view.unwrap().expiry_height,
                chain_id: v.body_view.unwrap().chain_id,
                fee: v
                    .body_view
                    .unwrap()
                    .fee
                    .ok_or_else(|| anyhow::anyhow!("missing fee field"))?
                    .try_into()?,
                fmd_clues: v
                    .body_view
                    .unwrap()
                    .fmd_clues
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                memo: v
                    .body_view
                    .unwrap()
                    .memo
                    .map(|m| MemoPlaintext::try_from(m.to_vec()))
                    .transpose()?,
                address_views: v
                    .body_view
                    .unwrap()
                    .address_views
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
            },
            binding_sig: v.binding_sig,
            anchor: v.anchor,
        })
    }
}

impl From<TransactionView> for pbt::TransactionView {
    fn from(v: TransactionView) -> Self {
        Self {
            body_view: Some(pbt::TransactionBodyView {
                action_views: v
                    .transaction_body_view
                    .action_views
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                expiry_height: v.transaction_body_view.expiry_height,
                chain_id: v.transaction_body_view.chain_id,
                fee: Some(v.transaction_body_view.fee.into()),
                fmd_clues: v
                    .transaction_body_view
                    .fmd_clues
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                memo: v.transaction_body_view.memo.map(|m| m.to_vec().into()),
                address_views: v
                    .transaction_body_view
                    .address_views
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }),
            binding_sig: v.binding_sig,
            anchor: v.anchor,
        }
    }
}
