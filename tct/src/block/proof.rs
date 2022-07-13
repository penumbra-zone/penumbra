use crate::prelude::*;

/// A proof of the inclusion of some [`Commitment`] in a [`Tree`] with a particular (non-global) [`Root`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(crate::internal::proof::Proof<frontier::Top<frontier::Item>>);

impl Proof {
    /// Construct a new [`Proof`] of inclusion for a given [`Commitment`], index, and authentication
    /// path from root to leaf.
    pub fn new(commitment: Commitment, block_position: u16, auth_path: [[Hash; 3]; 8]) -> Self {
        use crate::internal::path::{Leaf, Node};

        let position = block_position.into();

        let [a, b, c, d, e, f, g, h] = auth_path;
        let child = Leaf;
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

    /// Verify a [`Proof`] of inclusion against the [`Root`] of a [`Tree`].
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError`] if the proof is invalid for that [`Root`].
    pub fn verify(&self, root: super::Root) -> Result<(), VerifyError> {
        self.0.verify(root.0)
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn commitment(&self) -> Commitment {
        self.0.leaf
    }

    /// Get the position of the witnessed commitment.
    pub fn position(&self) -> crate::Position {
        self.0.index().into()
    }

    /// Get the authentication path for this proof, order from root to leaf.
    pub fn auth_path(&self) -> [&[Hash; 3]; 8] {
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
        let Leaf = child;
        [a, b, c, d, e, f, g, h]
    }
}

use penumbra_proto::crypto as pb;

impl From<Proof> for pb::NoteCommitmentBlockProof {
    fn from(proof: Proof) -> Self {
        proof.0.into()
    }
}

impl TryFrom<pb::NoteCommitmentBlockProof> for Proof {
    type Error = crate::error::proof::DecodeError;

    fn try_from(value: pb::NoteCommitmentBlockProof) -> Result<Self, Self::Error> {
        Ok(Proof(crate::internal::proof::Proof::try_from(value)?))
    }
}

impl penumbra_proto::Protobuf<pb::NoteCommitmentBlockProof> for Proof {}
