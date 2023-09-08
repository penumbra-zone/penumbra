use ark_ff::Zero;
use ark_secp256k1::Fr as Fsecp256k1;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress};
use hmac::{Hmac, Mac};

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

#[allow(non_snake_case)]
pub fn ckd_priv(k_par: [u8; 32], c_par: [u8; 32], i: u32) -> ([u8; 32], [u8; 32]) {
    let mut hmac = Hmac::<sha2::Sha512>::new_from_slice(&c_par).expect("can create hmac");
    if i >= 0x80000000 {
        // Hardened derivation
        hmac.update(&[0u8]);
        hmac.update(&k_par);
    } else {
        hmac.update(&k_par);
    }
    hmac.update(&i.to_be_bytes());

    // The output of the above hash is 64 bytes, and we split it into two 32 byte chunks, i_L and i_R.
    let result = hmac.finalize().into_bytes();
    let mut i_L = [0u8; 32];
    i_L.copy_from_slice(&result[..32]);
    let mut i_R = [0u8; 32];
    i_R.copy_from_slice(&result[32..]);

    // The result of the above is the child key k_i and the chain code c_i.
    let c_i = i_R;

    // k_i is derived as (k_par + i_L) % n
    // where n is the order of the secp256k1 curve:
    //
    // n = FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141
    //
    // which in the Arkworks crate is the scalar field modulus, in decimal:
    //
    // 115792089237316195423570985008687907852837564279074904382605163141518161494337
    //
    // See:
    // https://en.bitcoin.it/wiki/Secp256k1
    // https://en.bitcoin.it/wiki/BIP_0032#Child_key_derivation_.28CKD.29_functions
    let i_L_field = Fsecp256k1::deserialize_compressed(&i_L[..]).expect("valid Fr");
    let k_i = Fsecp256k1::deserialize_compressed(&k_par[..]).expect("valid Fr") + i_L_field;

    let mut i_L_mod_n_bytes = Vec::new();
    i_L_field
        .serialize_with_mode(&mut i_L_mod_n_bytes, Compress::Yes)
        .expect("can serialize");
    // Finally, we need to check if i_L ≥ n or k_i = 0, as the resulting key is invalid
    if k_i == Fsecp256k1::zero() || i_L_mod_n_bytes != i_L.to_vec() {
        // Key is invalid, proceed with the next value for i
        return ckd_priv(k_par, c_par, i + 1);
    }

    let mut k_i_bytes = Vec::new();
    k_i.serialize_with_mode(&mut k_i_bytes, Compress::Yes)
        .expect("can serialize");
    (k_i_bytes.try_into().expect("result fits in 32 bytes"), c_i)
}
