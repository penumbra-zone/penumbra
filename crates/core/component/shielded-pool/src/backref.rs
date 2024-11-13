use penumbra_keys::BackreferenceKey;
use penumbra_tct as tct;

pub const ENCRYPTED_BACKREF_LEN: usize = 48;

pub struct Backref {
    pub note_commitment: tct::StateCommitment,
}

#[derive(Clone, Debug)]
pub struct EncryptedBackref {
    pub bytes: Vec<u8>,
}
