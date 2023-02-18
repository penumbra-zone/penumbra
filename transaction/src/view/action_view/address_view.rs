use penumbra_crypto::{Note, PayloadKey};
use penumbra_proto::{core::transaction::v1alpha1 as pbt, DomainType};
use serde::{Deserialize, Serialize};

use crate::action::Output;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "pbt::AddressView", into = "pbt::AddressView")]
#[allow(clippy::large_enum_variant)]
pub enum AddressView {
    Opaque(Address),
    Visible {
        address: Address,
        index: AddressIndex,
    },
}

impl DomainType for AddressView {
    type Proto = pbt::OutputView;
}

impl TryFrom<pbt::AddressView> for AddressView {
    type Error = anyhow::Error;

    fn try_from(v: pbt::AddressView) -> Result<Self, Self::Error> {
        match v
            .address_view
            .ok_or_else(|| anyhow::anyhow!("missing address field"))?
        {
            pbt::address_view::AddressView::Visible(x) => Ok(AddressView::Visible {
                address: x
                    .address
                    .ok_or_else(|| anyhow::anyhow!("missing address field"))?
                    .try_into()?,
                index: x
                    .index
                    .ok_or_else(|| anyhow::anyhow!("missing address field"))?
                    .try_into()?,
            }),
            pbt::address_view::AddressView::Opaque(x) => Ok(AddressView::Opaque {
                address: x
                    .address
                    .ok_or_else(|| anyhow::anyhow!("missing address field"))?
                    .try_into()?,
            }),
        }
    }
}

impl From<AddressView> for pbt::AddressView {
    fn from(v: AddressView) -> Self {
        use pbt::address_view as av;
        match v {
            AddressView::Visible {} => Self {
                address_view: Some(ov::AddressView::Visible(av::Visible {})),
            },
            AddressView::Opaque {} => Self {
                address_view: Some(ov::AddressView::Opaque(av::Opaque {})),
            },
        }
    }
}
