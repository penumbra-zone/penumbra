pub use thiserror::Error;

use crate::{Commitment, Hash, VerifyError};

pub use super::{Eternity, Position, Root};

/// An as-yet-unverified proof of the inclusion of some [`Commitment`] in an [`Eternity`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(pub(super) crate::proof::Proof<Eternity>);

impl Proof {
    /// Construct a new [`Proof`] of inclusion for a given [`Commitment`], index, and authentication
    /// path from root to leaf.
    pub fn new(
        commitment: Commitment,
        Position(index): Position,
        auth_path: [[Hash; 3]; 24],
    ) -> Self {
        use crate::internal::path::{Leaf, Node};
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x] = auth_path;
        let path = Leaf;
        let path = Node {
            siblings: x,
            child: path,
        };
        let path = Node {
            siblings: w,
            child: path,
        };
        let path = Node {
            siblings: v,
            child: path,
        };
        let path = Node {
            siblings: u,
            child: path,
        };
        let path = Node {
            siblings: t,
            child: path,
        };
        let path = Node {
            siblings: s,
            child: path,
        };
        let path = Node {
            siblings: r,
            child: path,
        };
        let path = Node {
            siblings: q,
            child: path,
        };
        let path = Node {
            siblings: p,
            child: path,
        };
        let path = Node {
            siblings: o,
            child: path,
        };
        let path = Node {
            siblings: n,
            child: path,
        };
        let path = Node {
            siblings: m,
            child: path,
        };
        let path = Node {
            siblings: l,
            child: path,
        };
        let path = Node {
            siblings: k,
            child: path,
        };
        let path = Node {
            siblings: j,
            child: path,
        };
        let path = Node {
            siblings: i,
            child: path,
        };
        let path = Node {
            siblings: h,
            child: path,
        };
        let path = Node {
            siblings: g,
            child: path,
        };
        let path = Node {
            siblings: f,
            child: path,
        };
        let path = Node {
            siblings: e,
            child: path,
        };
        let path = Node {
            siblings: d,
            child: path,
        };
        let path = Node {
            siblings: c,
            child: path,
        };
        let path = Node {
            siblings: b,
            child: path,
        };
        let path = Node {
            siblings: a,
            child: path,
        };
        Self(crate::proof::Proof {
            leaf: commitment,
            position: index.into(),
            auth_path: path,
        })
    }

    /// Verify a [`Proof`] of inclusion against the [`Root`] of an [`Eternity`].
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError`] if the proof is invalid for that [`Root`].
    pub fn verify(&self, root: Root) -> Result<(), VerifyError> {
        self.0.verify(root.0)
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn commitment(&self) -> Commitment {
        self.0.leaf
    }

    /// Get the position of the witnessed commitment.
    pub fn position(&self) -> crate::Position {
        crate::eternity::Position(self.0.index().into())
    }

    /// Get the authentication path for this proof, order from root to leaf.
    pub fn auth_path(&self) -> [&[Hash; 3]; 24] {
        use crate::internal::path::{Leaf, Node};
        let path = self.0.auth_path();
        let Node {
            siblings: a,
            child: path,
        } = path;
        let Node {
            siblings: b,
            child: path,
        } = path;
        let Node {
            siblings: c,
            child: path,
        } = path;
        let Node {
            siblings: d,
            child: path,
        } = path;
        let Node {
            siblings: e,
            child: path,
        } = path;
        let Node {
            siblings: f,
            child: path,
        } = path;
        let Node {
            siblings: g,
            child: path,
        } = path;
        let Node {
            siblings: h,
            child: path,
        } = path;
        let Node {
            siblings: i,
            child: path,
        } = path;
        let Node {
            siblings: j,
            child: path,
        } = path;
        let Node {
            siblings: k,
            child: path,
        } = path;
        let Node {
            siblings: l,
            child: path,
        } = path;
        let Node {
            siblings: m,
            child: path,
        } = path;
        let Node {
            siblings: n,
            child: path,
        } = path;
        let Node {
            siblings: o,
            child: path,
        } = path;
        let Node {
            siblings: p,
            child: path,
        } = path;
        let Node {
            siblings: q,
            child: path,
        } = path;
        let Node {
            siblings: r,
            child: path,
        } = path;
        let Node {
            siblings: s,
            child: path,
        } = path;
        let Node {
            siblings: t,
            child: path,
        } = path;
        let Node {
            siblings: u,
            child: path,
        } = path;
        let Node {
            siblings: v,
            child: path,
        } = path;
        let Node {
            siblings: w,
            child: path,
        } = path;
        let Node {
            siblings: x,
            child: path,
        } = path;
        let Leaf = path;
        [
            a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x,
        ]
    }
}

// TODO: re-enable these impls once we adapt the protobuf crate:

/*
use penumbra_proto::transparent_proofs as pb;

impl From<Proof> for pb::MerkleProof {
    fn from(proof: Proof) -> Self {
        proof.0.into()
    }
}

impl TryFrom<pb::MerkleProof> for Proof {
    type Error = crate::ProofDecodeError;

    fn try_from(value: pb::MerkleProof) -> Result<Self, Self::Error> {
        Ok(Proof(crate::internal::proof::Proof::try_from(value)?))
    }
}

impl penumbra_proto::Protobuf<pb::MerkleProof> for Proof {}
*/
