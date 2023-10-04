/// Penumbra's registered coin type.
/// See: https://github.com/satoshilabs/slips/pull/1592
const PENUMBRA_COIN_TYPE: u32 = 6532;

/// Represents a BIP44 derivation path.
///
/// BIP43 defines the purpose constant used for BIP44 derivation.
///
/// BIP43: https://github.com/bitcoin/bips/blob/master/bip-0043.mediawiki
/// BIP44: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
pub struct Bip44Path {
    purpose: u32,
    coin_type: u32,
    account: u32,
    change: Option<u32>,
    address_index: Option<u32>,
}

impl Bip44Path {
    /// Create a new BIP44 path for a Penumbra wallet.
    pub fn new(account: u32) -> Self {
        Self {
            purpose: 44,
            coin_type: PENUMBRA_COIN_TYPE,
            account,
            change: None,
            address_index: None,
        }
    }

    /// Create a new generic BIP44 path.
    pub fn new_generic(
        purpose: u32,
        coin_type: u32,
        account: u32,
        change: Option<u32>,
        address_index: Option<u32>,
    ) -> Self {
        Self {
            purpose,
            coin_type,
            account,
            change,
            address_index,
        }
    }

    /// Per BIP43, purpose is typically a constant set to 44' or 0x8000002C.
    pub fn purpose(&self) -> u32 {
        self.purpose
    }

    /// Per BIP44, coin type is a constant set for each currency.
    pub fn coin_type(&self) -> u32 {
        self.coin_type
    }

    /// Per BIP44, account splits the key space into independent user identities.
    pub fn account(&self) -> u32 {
        self.account
    }

    /// Change is set to 1 to denote change addresses. None if unset.
    pub fn change(&self) -> Option<u32> {
        self.change
    }

    /// Addresses are numbered starting from index 0. None if unset.
    pub fn address_index(&self) -> Option<u32> {
        self.address_index
    }

    pub fn path(&self) -> String {
        let mut path = format!("m/44'/{}'/{}'", self.coin_type(), self.account());
        if self.change().is_some() {
            path = format!("{}/{}", path, self.change().expect("change will exist"));
        }
        if self.address_index().is_some() {
            path = format!(
                "{}/{}",
                path,
                self.address_index().expect("address index will exist")
            );
        }
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip44_path_full() {
        let path = Bip44Path::new_generic(44, 6532, 0, Some(0), Some(0));
        assert_eq!(path.path(), "m/44'/6532'/0'/0/0");
    }

    #[test]
    fn test_bip44_path_account_level() {
        let path = Bip44Path::new(0);
        assert_eq!(path.path(), "m/44'/6532'/0'");
    }
}
