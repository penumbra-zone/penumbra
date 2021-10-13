mod diversifier;
pub use diversifier::{Diversifier, DIVERSIFIER_LEN_BYTES};

mod nullifier;
pub use nullifier::{NullifierDerivingKey, NullifierPrivateKey, NK_LEN_BYTES};

mod proof;
pub use proof::ProofAuthKey;

mod spend;
pub use spend::{SpendKey, SpendSeed, SPENDSEED_LEN_BYTES};

mod view;
pub use view::{FullViewingKey, OutgoingViewingKey, IVK_LEN_BYTES, OVK_LEN_BYTES};

#[cfg(test)]
mod tests {
    use super::*;

    use rand_core::OsRng;

    use crate::addresses::PaymentAddress;

    // Remove this when the code is more complete; it's just
    // a way to exercise the codepaths.
    #[test]
    fn scratch_complete_key_generation_happy_path() {
        let mut rng = OsRng;
        let diversifier = Diversifier::generate(&mut rng);
        let sk = SpendKey::generate(&mut rng);
        let _proof_auth_key = sk.proof_authorization_key();
        let fvk = sk.full_viewing_key();
        let ivk = fvk.incoming();
        let _dest = PaymentAddress::new(ivk, diversifier);
    }
}
