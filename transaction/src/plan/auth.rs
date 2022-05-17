use penumbra_crypto::keys::SpendKey;
use rand::{CryptoRng, RngCore};

use crate::{plan::TransactionPlan, AuthorizationData};

impl TransactionPlan {
    /// Authorize this [`TransactionPlan`] with the provided [`SpendKey`].
    ///
    /// The returned [`AuthorizationData`] can be used to build a [`Transaction`](crate::Transaction).
    pub fn authorize<R: RngCore + CryptoRng>(
        &self,
        mut rng: R,
        sk: &SpendKey,
    ) -> AuthorizationData {
        let auth_hash = self.auth_hash(sk.full_viewing_key());
        let mut spend_auths = Vec::new();
        for spend_plan in self.spend_plans() {
            let rsk = sk.spend_auth_key().randomize(&spend_plan.randomizer);
            let auth_sig = rsk.sign(&mut rng, auth_hash.as_ref());
            spend_auths.push(auth_sig);
        }
        AuthorizationData {
            auth_hash,
            spend_auths,
        }
    }
}
