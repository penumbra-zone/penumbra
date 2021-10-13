use super::NullifierPrivateKey;
use crate::rdsa::{SpendAuth, VerificationKey};

pub struct ProofAuthKey {
    pub ak: VerificationKey<SpendAuth>,
    pub nsk: NullifierPrivateKey,
}
