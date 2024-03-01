pub struct Keys {
    pub consensus_signing_key: ed25519_consensus::SigningKey,
    pub consensus_verification_key: ed25519_consensus::VerificationKey,
}

impl Keys {
    pub fn generate() -> Self {
        Self::generate_with(rand_core::OsRng)
    }

    /// Generates  a set of keys using the provided random number generator.
    pub fn generate_with<R>(rng: R) -> Self
    where
        R: rand_core::RngCore + rand_core::CryptoRng,
    {
        let consensus_signing_key = ed25519_consensus::SigningKey::new(rng);
        let consensus_verification_key = consensus_signing_key.verification_key();

        Self {
            consensus_signing_key,
            consensus_verification_key,
        }
    }
}
