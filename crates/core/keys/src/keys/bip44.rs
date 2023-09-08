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
    purpose: u32,
    coin_type: u32,
    account: u32,
    change: u32,
    address_index: u32,
}

impl Bip44Path {
    /// Create a new BIP44 path for Penumbra.
    pub fn new(account: u32, change: u32, address_index: u32) -> Self {
        Self {
            purpose: 0x8000002C,
            coin_type: PENUMBRA_COIN_TYPE,
            account,
            change,
            address_index,
        }
    }

    /// Create a new generic BIP44 path.
    pub fn new_generic(
        purpose: u32,
        coin_type: u32,
        account: u32,
        change: u32,
        address_index: u32,
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
        self.coin_type | 0x80000000
    }

    /// Per BIP44, account splits the key space into independent user identities.
    pub fn account(&self) -> u32 {
        self.account | 0x80000000
    }

    /// Change is set to 1 to denote change addresses.
    pub fn change(&self) -> u32 {
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
            self.change(),
            self.address_index()
        )
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
        dbg!("this should only occur with w/ low probability");
        dbg!(k_i);
        dbg!(i_L_mod_n_bytes);
        dbg!(i_L.to_vec());
        return ckd_priv(k_par, c_par, i + 1);
    }

    let mut k_i_bytes = Vec::new();
    k_i.serialize_with_mode(&mut k_i_bytes, Compress::Yes)
        .expect("can serialize");
    (k_i_bytes.try_into().expect("result fits in 32 bytes"), c_i)
}

#[cfg(test)]
mod tests {
    use bs58;

    use super::*;

    #[test]
    fn test_bip44_path() {
        let path = Bip44Path::new(0, 0, 0);
        assert_eq!(path.path(), "m/44'/6532'/0'/0/0");
    }

    /// The below test vectors are from: https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki#test-vector-1
    #[test]
    fn bip44_child_derivation() {
        // Test vector 1
        let hex_seed = "000102030405060708090a0b0c0d0e0f".to_string();
        let seed = hex::decode(hex_seed).expect("can decode test vector");
        let mut seed_arr = [0u8; 32];
        seed_arr[0..16].copy_from_slice(&seed[..]);

        let path = Bip44Path::new_generic(0, 1, 2, 2, 1000000000);

        // Chain m
        let data = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi".to_string();
        let decoded_bytes = bs58::decode(data)
            .into_vec()
            .expect("all test vectors should be valid base58");

        // Per BIP43, each address has the prefix 0x0488B21E for public and 0x0488ADE4 for private nodes.
        // All our test vectors are private nodes since we only implemented the private key derivation:
        assert_eq!(decoded_bytes[0..4], [0x04, 0x88, 0xAD, 0xE4]);

        // Extended keys are 82 bytes long, the final 4 bytes being a checksum, and the preceding 33 bytes being the
        // actual key material. Public keys are 33 bytes, and private keys get a 0x00 appended since they are 32 bytes.
        let priv_key_material = &decoded_bytes[46..78];

        // First level is hardened derivation in test vector 1.
        let i = 2147483648u32; // 2^31
        let mut purpose_bytes = [0u8; 32];
        let purpose = path.purpose();
        let purpose_arr = path.purpose().to_le_bytes();
        purpose_bytes[0..4].copy_from_slice(&purpose_arr[..]);
        let (k_1, _) = ckd_priv(seed_arr, purpose_bytes, i);

        assert_eq!(
            k_1, priv_key_material,
            "first level derivation should match"
        );
    }
}
