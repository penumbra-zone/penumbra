/// A hash of a transaction's _authorizing data_, describing both its effects on
/// the chain state as well as the cryptographic authorization of those effects.
///
/// In practice this is simply a hash of the `TransactionBody`.
///
/// The transaction's binding signature is formed over the transaction's
/// `AuthHash`.  Because the binding signature is formed using the randomness
/// for the balance commitments for each action, this prevents replaying proofs
/// from one transaction to another without knowledge of the openings of the
/// balance commitments, binding the proofs to the transaction they were
/// originally computed for.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AuthHash(pub [u8; 32]);

impl std::fmt::Debug for AuthHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AuthHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl AuthHash {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

pub trait AuthorizingData {
    fn auth_hash(&self) -> AuthHash;
}
