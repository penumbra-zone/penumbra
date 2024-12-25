use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use penumbra_sdk_proto::{penumbra::core::keys::v1 as pb, DomainType};

use crate::keys::{AddressIndex, WalletId};

use super::Address;

/// A view of a Penumbra address, either an opaque payment address or an address
/// with known structure.
///
/// This type allows working with addresses and address indexes without knowing
/// the corresponding FVK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::AddressView", into = "pb::AddressView")]
pub enum AddressView {
    Opaque {
        address: Address,
    },
    Decoded {
        address: Address,
        index: AddressIndex,
        wallet_id: WalletId,
    },
}

impl AddressView {
    pub fn address(&self) -> Address {
        match self {
            AddressView::Opaque { address } => address.clone(),
            AddressView::Decoded { address, .. } => address.clone(),
        }
    }
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
            AddressView::Decoded {
                address,
                index,
                wallet_id,
            } => Self {
                address_view: Some(pb::address_view::AddressView::Decoded(
                    pb::address_view::Decoded {
                        address: Some(address.into()),
                        index: Some(index.into()),
                        wallet_id: Some(wallet_id.into()),
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
            Some(pb::address_view::AddressView::Decoded(visible)) => {
                let address = visible
                    .address
                    .ok_or_else(|| anyhow::anyhow!("AddressView::Visible missing address field"))?
                    .try_into()?;
                let index = visible
                    .index
                    .ok_or_else(|| anyhow::anyhow!("AddressView::Visible missing index field"))?
                    .try_into()?;
                let wallet_id = visible
                    .wallet_id
                    .ok_or_else(|| anyhow::anyhow!("AddressView::Visible missing wallet_id field"))?
                    .try_into()?;
                Ok(AddressView::Decoded {
                    address,
                    index,
                    wallet_id,
                })
            }
            None => Err(anyhow::anyhow!("AddressView missing address_view field")),
        }
    }
}

// Canonical ordering for serialization
impl PartialOrd for AddressView {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use AddressView::*;
        // Opaque < Decoded
        match (self, other) {
            (Opaque { .. }, Decoded { .. }) => Some(Ordering::Less),
            (Decoded { .. }, Opaque { .. }) => Some(Ordering::Greater),
            (Opaque { address: a1 }, Opaque { address: a2 }) => a1.partial_cmp(a2),
            (
                Decoded {
                    address: a1,
                    index: i1,
                    wallet_id: w1,
                },
                Decoded {
                    address: a2,
                    index: i2,
                    wallet_id: w2,
                },
            ) => (a1, i1, w1).partial_cmp(&(a2, i2, w2)),
        }
    }
}

impl Ord for AddressView {
    fn cmp(&self, other: &Self) -> Ordering {
        // Opaque < Decoded
        match (self, other) {
            (AddressView::Opaque { address: a1 }, AddressView::Opaque { address: a2 }) => {
                a1.cmp(a2)
            }
            (
                AddressView::Decoded {
                    address: a1,
                    index: i1,
                    wallet_id: w1,
                },
                AddressView::Decoded {
                    address: a2,
                    index: i2,
                    wallet_id: w2,
                },
            ) => match a1.cmp(a2) {
                Ordering::Equal => match i1.cmp(i2) {
                    Ordering::Equal => w1.cmp(w2),
                    ord => ord,
                },
                ord => ord,
            },
            (
                AddressView::Opaque { address: _ },
                AddressView::Decoded {
                    address: _,
                    index: _,
                    wallet_id: _,
                },
            ) => Ordering::Less,
            (
                AddressView::Decoded {
                    address: _,
                    index: _,
                    wallet_id: _,
                },
                AddressView::Opaque { address: _ },
            ) => Ordering::Greater,
        }
    }
}

#[cfg(test)]
mod tests {
    use rand_core::OsRng;

    use crate::keys::{Bip44Path, SeedPhrase, SpendKey};

    use super::*;

    #[test]
    fn address_view_basic() {
        let sk1 = SpendKey::from_seed_phrase_bip44(SeedPhrase::generate(OsRng), &Bip44Path::new(0));
        let sk2 = SpendKey::from_seed_phrase_bip44(SeedPhrase::generate(OsRng), &Bip44Path::new(0));

        let fvk1 = sk1.full_viewing_key();
        let fvk2 = sk2.full_viewing_key();

        let addr1_0 = fvk1.payment_address(0.into()).0;
        let addr1_1 = fvk1.payment_address(1.into()).0;
        let addr2_0 = fvk2.payment_address(0.into()).0;
        let addr2_1 = fvk2.payment_address(1.into()).0;

        assert_eq!(
            fvk1.view_address(addr1_0.clone()),
            AddressView::Decoded {
                address: addr1_0.clone(),
                index: 0.into(),
                wallet_id: fvk1.wallet_id(),
            }
        );
        assert_eq!(
            fvk2.view_address(addr1_0.clone()),
            AddressView::Opaque {
                address: addr1_0.clone()
            }
        );
        assert_eq!(
            fvk1.view_address(addr1_1.clone()),
            AddressView::Decoded {
                address: addr1_1.clone(),
                index: 1.into(),
                wallet_id: fvk1.wallet_id(),
            }
        );
        assert_eq!(
            fvk2.view_address(addr1_1.clone()),
            AddressView::Opaque {
                address: addr1_1.clone()
            }
        );
        assert_eq!(
            fvk1.view_address(addr2_0.clone()),
            AddressView::Opaque {
                address: addr2_0.clone()
            }
        );
        assert_eq!(
            fvk2.view_address(addr2_0.clone()),
            AddressView::Decoded {
                address: addr2_0.clone(),
                index: 0.into(),
                wallet_id: fvk2.wallet_id(),
            }
        );
        assert_eq!(
            fvk1.view_address(addr2_1.clone()),
            AddressView::Opaque {
                address: addr2_1.clone()
            }
        );
        assert_eq!(
            fvk2.view_address(addr2_1.clone()),
            AddressView::Decoded {
                address: addr2_1.clone(),
                index: 1.into(),
                wallet_id: fvk2.wallet_id(),
            }
        );
    }
}
