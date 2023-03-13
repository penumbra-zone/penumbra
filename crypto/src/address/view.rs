use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType};
use serde::{Deserialize, Serialize};

use crate::keys::{AccountGroupId, AddressIndex};

use super::Address;

/// A view of a Penumbra address, either an opaque payment address or an address
/// with known structure.
///
/// This type allows working with addresses and address indexes without knowing
/// the corresponding FVK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::AddressView", into = "pb::AddressView")]
pub enum AddressView {
    Opaque {
        address: Address,
    },
    Visible {
        address: Address,
        index: AddressIndex,
        account_group_id: AccountGroupId,
    },
}

impl DomainType for AddressView {
    type Proto = pb::AddressView;
}

impl From<AddressView> for pb::AddressView {
    fn from(view: AddressView) -> Self {
        match view {
            AddressView::Opaque { address } => Self {
                address_view: Some(pb::address_view::AddressView::Opaque(
                    pb::address_view::Opaque {
                        address: Some(address.into()),
                    },
                )),
            },
            AddressView::Visible {
                address,
                index,
                account_group_id,
            } => Self {
                address_view: Some(pb::address_view::AddressView::Visible(
                    pb::address_view::Visible {
                        address: Some(address.into()),
                        index: Some(index.into()),
                        account_group_id: Some(account_group_id.into()),
                    },
                )),
            },
        }
    }
}

impl TryFrom<pb::AddressView> for AddressView {
    type Error = anyhow::Error;
    fn try_from(value: pb::AddressView) -> Result<Self, Self::Error> {
        match value.address_view {
            Some(pb::address_view::AddressView::Opaque(opaque)) => {
                let address = opaque
                    .address
                    .ok_or_else(|| anyhow::anyhow!("AddressView::Opaque missing address field"))?
                    .try_into()?;
                Ok(AddressView::Opaque { address })
            }
            Some(pb::address_view::AddressView::Visible(visible)) => {
                let address = visible
                    .address
                    .ok_or_else(|| anyhow::anyhow!("AddressView::Visible missing address field"))?
                    .try_into()?;
                let index = visible
                    .index
                    .ok_or_else(|| anyhow::anyhow!("AddressView::Visible missing index field"))?
                    .try_into()?;
                let account_group_id = visible
                    .account_group_id
                    .ok_or_else(|| {
                        anyhow::anyhow!("AddressView::Visible missing account_group_id field")
                    })?
                    .try_into()?;
                Ok(AddressView::Visible {
                    address,
                    index,
                    account_group_id,
                })
            }
            None => Err(anyhow::anyhow!("AddressView missing address_view field")),
        }
    }
}
