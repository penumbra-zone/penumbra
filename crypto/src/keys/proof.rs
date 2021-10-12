use super::{NullifierPrivateKey, SpendAuth, VerificationKey};

pub struct ProofAuthorizationKey {
    pub ak: VerificationKey<SpendAuth>,
    pub nsk: NullifierPrivateKey,
}
