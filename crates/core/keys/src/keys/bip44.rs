/// Penumbra's registered coin type.
/// See: https://github.com/satoshilabs/slips/pull/1592
const PENUMBRA_COIN_TYPE: u32 = 0x0001984;

/// Represents a BIP44 derivation path.
///
/// BIP43 defines the purpose constant used for BIP44 derivation.
///
/// BIP43: https://github.com/bitcoin/bips/blob/master/bip-0043.mediawiki
/// BIP44: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
pub struct Bip44Path {
    coin_type: u32,
    account: u32,
    change: bool,
    address_index: u32,
}

impl Bip44Path {
    /// Create a new BIP44 path for Penumbra.
    pub fn new(account: u32, change: bool, address_index: u32) -> Self {
        Self {
            coin_type: PENUMBRA_COIN_TYPE,
            account,
            change,
            address_index,
        }
    }

    /// Per BIP43, purpose is a constant set to 44' or 0x8000002C.
    pub fn purpose(&self) -> u32 {
        0x8000002C
    }

    /// Per BIP44, coin type is a constant set for each currency.
    pub fn coin_type(&self) -> u32 {
        self.coin_type
    }

    /// Per BIP44, account splits the key space into independent user identities.
    pub fn account(&self) -> u32 {
        self.account
    }

    /// Change is set to 1 to denote change addresses.
    pub fn change(&self) -> bool {
        self.change
    }

    /// Addresses are numbered starting from index 0.
    pub fn address_index(&self) -> u32 {
        self.address_index
    }

    pub fn path(&self) -> String {
        format!(
            "m/44'/{}'/{}'/{}/{}",
            self.coin_type(),
            self.account(),
            if self.change() { 1 } else { 0 },
            self.address_index()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bip44_path() {
        let path = Bip44Path::new(0, false, 0);
        assert_eq!(path.path(), "m/44'/6532'/0'/0/0");
    }
}
