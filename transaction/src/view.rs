use decaf377_fmd::Clue;
use penumbra_crypto::{memo::MemoPlaintext, transaction::Fee};
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};
use serde::{Deserialize, Serialize};

pub mod action_view;
mod transaction_perspective;

pub use action_view::ActionView;
pub use transaction_perspective::TransactionPerspective;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::TransactionView", into = "pbt::TransactionView")]
pub struct TransactionView {
    pub action_views: Vec<ActionView>,
    pub expiry_height: u64,
    pub chain_id: String,
    pub fee: Fee,
    pub fmd_clues: Vec<Clue>,
    pub memo: Option<MemoPlaintext>,
}

impl DomainType for TransactionView {
    type Proto = pbt::TransactionView;
}

impl TryFrom<pbt::TransactionView> for TransactionView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::TransactionView) -> Result<Self, Self::Error> {
        Ok(Self {
            action_views: v
                .action_views
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            expiry_height: v.expiry_height,
            chain_id: v.chain_id,
            fee: v
                .fee
                .ok_or_else(|| anyhow::anyhow!("missing fee field"))?
                .try_into()?,
            fmd_clues: v
                .fmd_clues
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            memo: v
                .memo
                .map(|m| MemoPlaintext::try_from(m.to_vec()))
                .transpose()?,
        })
    }
}

impl From<TransactionView> for pbt::TransactionView {
    fn from(v: TransactionView) -> Self {
        Self {
            action_views: v.action_views.into_iter().map(Into::into).collect(),
            expiry_height: v.expiry_height,
            chain_id: v.chain_id,
            fee: Some(v.fee.into()),
            fmd_clues: v.fmd_clues.into_iter().map(Into::into).collect(),
            memo: v.memo.map(|m| m.to_vec().into()),
        }
    }
}
