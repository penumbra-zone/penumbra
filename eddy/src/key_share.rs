/// A decryptor's private key share.
pub struct PrivateKeyShare {
    pub(crate) participant_index: u32,
    pub(crate) key_share: decaf377::Fr,
    pub(crate) cached_pub: PublicKeyShare,
}

/// A decryptor's public key share.
pub struct PublicKeyShare {
    pub(crate) participant_index: u32,
    pub(crate) pub_key_share: decaf377::Element,
}
