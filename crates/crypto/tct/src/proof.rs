use poseidon377::Fq;

use crate::prelude::*;

/// A proof of the inclusion of some [`Commitment`] in a [`Tree`] with a particular [`Root`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(
    pub(super)  crate::internal::proof::Proof<
        frontier::Top<frontier::Tier<frontier::Tier<frontier::Item>>>,
    >,
);

impl Proof {
    /// Construct a new [`Proof`] of inclusion for a given [`Commitment`], index, and authentication
    /// path from root to leaf.
    pub fn new(
        commitment: StateCommitment,
        position: Position,
        auth_path: [[Hash; 3]; 24],
    ) -> Self {
        use crate::internal::path::{Leaf, Node};

        let position = position.into();

        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x] = auth_path;
        let child = Leaf;
        let child = Node { siblings: x, child };
        let child = Node { siblings: w, child };
        let child = Node { siblings: v, child };
        let child = Node { siblings: u, child };
        let child = Node { siblings: t, child };
        let child = Node { siblings: s, child };
        let child = Node { siblings: r, child };
        let child = Node { siblings: q, child };
        let child = Node { siblings: p, child };
        let child = Node { siblings: o, child };
        let child = Node { siblings: n, child };
        let child = Node { siblings: m, child };
        let child = Node { siblings: l, child };
        let child = Node { siblings: k, child };
        let child = Node { siblings: j, child };
        let child = Node { siblings: i, child };
        let child = Node { siblings: h, child };
        let child = Node { siblings: g, child };
        let child = Node { siblings: f, child };
        let child = Node { siblings: e, child };
        let child = Node { siblings: d, child };
        let child = Node { siblings: c, child };
        let child = Node { siblings: b, child };
        let child = Node { siblings: a, child };
        Self(crate::internal::proof::Proof {
            leaf: commitment,
            position,
            auth_path: child,
        })
    }

    /// Generate a dummy [`Proof`] for a given commitment.
    pub fn dummy<R: Rng + rand::CryptoRng>(rng: &mut R, commitment: StateCommitment) -> Self {
        let dummy_position = 0u64.into();
        let dummy_auth_path: [[Hash; 3]; 24] = [[Hash::new(Fq::rand(rng)); 3]; 24];
        Self::new(commitment, dummy_position, dummy_auth_path)
    }

    /// Verify a [`Proof`] of inclusion against the [`Root`] of a [`Tree`].
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError`] if the proof is invalid for that [`Root`].
    pub fn verify(&self, root: Root) -> Result<(), VerifyError> {
        self.0.verify(root.0)
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn commitment(&self) -> StateCommitment {
        self.0.leaf
    }

    /// Get the position of the witnessed commitment.
    pub fn position(&self) -> crate::Position {
        self.0.index().into()
    }

    /// Get the root of the tree from which the proof was generated.
    pub fn root(&self) -> Root {
        Root(self.0.root())
    }

    /// Get the authentication path for this proof, order from root to leaf.
    pub fn auth_path(&self) -> [&[Hash; 3]; 24] {
        use crate::internal::path::{Leaf, Node};
        let child = self.0.auth_path();
        let Node { siblings: a, child } = child;
        let Node { siblings: b, child } = child;
        let Node { siblings: c, child } = child;
        let Node { siblings: d, child } = child;
        let Node { siblings: e, child } = child;
        let Node { siblings: f, child } = child;
        let Node { siblings: g, child } = child;
        let Node { siblings: h, child } = child;
        let Node { siblings: i, child } = child;
        let Node { siblings: j, child } = child;
        let Node { siblings: k, child } = child;
        let Node { siblings: l, child } = child;
        let Node { siblings: m, child } = child;
        let Node { siblings: n, child } = child;
        let Node { siblings: o, child } = child;
        let Node { siblings: p, child } = child;
        let Node { siblings: q, child } = child;
        let Node { siblings: r, child } = child;
        let Node { siblings: s, child } = child;
        let Node { siblings: t, child } = child;
        let Node { siblings: u, child } = child;
        let Node { siblings: v, child } = child;
        let Node { siblings: w, child } = child;
        let Node { siblings: x, child } = child;
        let Leaf = child;
        [
            a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
        ]
    }
}

use penumbra_proto::penumbra::crypto::tct::v1 as pb;
use rand::Rng;

impl From<Proof> for pb::StateCommitmentProof {
    fn from(proof: Proof) -> Self {
        proof.0.into()
    }
}

impl TryFrom<pb::StateCommitmentProof> for Proof {
    type Error = crate::error::proof::DecodeError;

    fn try_from(value: pb::StateCommitmentProof) -> Result<Self, Self::Error> {
        Ok(Proof(crate::internal::proof::Proof::try_from(value)?))
    }
}

impl penumbra_proto::DomainType for Proof {
    type Proto = pb::StateCommitmentProof;
}
