use penumbra_proto::{core::crypto::v1alpha1 as pb, DomainType, TypeUrl};
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

impl AddressView {
    pub fn address(&self) -> Address {
        match self {
            AddressView::Opaque { address } => *address,
            AddressView::Visible { address, .. } => *address,
        }
    }
}

impl TypeUrl for AddressView {
    const TYPE_URL: &'static str = "/penumbra.core.crypto.v1alpha1.AddressView";
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{SeedPhrase, SpendKey};
    use rand_core::OsRng;

    #[test]
    fn address_view_basic() {
        let sk1 = SpendKey::from_seed_phrase(SeedPhrase::generate(OsRng), 0);
        let sk2 = SpendKey::from_seed_phrase(SeedPhrase::generate(OsRng), 0);

        let fvk1 = sk1.full_viewing_key();
        let fvk2 = sk2.full_viewing_key();

        let addr1_0 = fvk1.payment_address(0.into()).0;
        let addr1_1 = fvk1.payment_address(1.into()).0;
        let addr2_0 = fvk2.payment_address(0.into()).0;
        let addr2_1 = fvk2.payment_address(1.into()).0;

        assert_eq!(
            fvk1.view_address(addr1_0),
            AddressView::Visible {
                address: addr1_0,
                index: 0.into(),
                account_group_id: fvk1.account_group_id(),
            }
        );
        assert_eq!(
            fvk2.view_address(addr1_0),
            AddressView::Opaque { address: addr1_0 }
        );
        assert_eq!(
            fvk1.view_address(addr1_1),
            AddressView::Visible {
                address: addr1_1,
                index: 1.into(),
                account_group_id: fvk1.account_group_id(),
            }
        );
        assert_eq!(
            fvk2.view_address(addr1_1),
            AddressView::Opaque { address: addr1_1 }
        );
        assert_eq!(
            fvk1.view_address(addr2_0),
            AddressView::Opaque { address: addr2_0 }
        );
        assert_eq!(
            fvk2.view_address(addr2_0),
            AddressView::Visible {
                address: addr2_0,
                index: 0.into(),
                account_group_id: fvk2.account_group_id(),
            }
        );
        assert_eq!(
            fvk1.view_address(addr2_1),
            AddressView::Opaque { address: addr2_1 }
        );
        assert_eq!(
            fvk2.view_address(addr2_1),
            AddressView::Visible {
                address: addr2_1,
                index: 1.into(),
                account_group_id: fvk2.account_group_id(),
            }
        );
    }
}
