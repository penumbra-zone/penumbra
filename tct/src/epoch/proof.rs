pub use thiserror::Error;

use crate::{Commitment, Hash};

pub use super::{Epoch, Root};

/// An as-yet-unverified proof of the inclusion of some [`Commitment`] in an [`Epoch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof(pub(super) crate::proof::Proof<Epoch>);

impl Proof {
    /// Construct a new [`Proof`] of inclusion for a given [`Commitment`], index, and authentication
    /// path from root to leaf.
    pub fn new(commitment: Commitment, index: u64, auth_path: [[Hash; 3]; 16]) -> Self {
        use crate::internal::path::{Leaf, Node};
        let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = auth_path;
        let path = Leaf;
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
            index,
            auth_path: path,
        })
    }

    /// Verify a [`Proof`] of inclusion against the [`Root`] of an [`Epoch`].
    ///
    ///
    /// # Errors
    ///
    /// Returns [`VerifyError`] if the proof is invalid for that [`Root`].
    pub fn verify(self, root: &Root) -> Result<(), VerifyError> {
        self.0.verify(root.0).map_err(VerifyError).map(|_| ())
    }

    /// Get the commitment whose inclusion is witnessed by the proof.
    pub fn commitment(&self) -> Commitment {
        self.0.leaf
    }

    /// Get the index of the witnessed commitment.
    pub fn index(&self) -> u64 {
        self.0.index()
    }

    /// Get the authentication path for this proof, order from root to leaf.
    pub fn auth_path(&self) -> [&[Hash; 3]; 16] {
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
        let Leaf = path;
        [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p]
    }
}

/// A [`Proof`] of inclusion did not verify against the provided root of the [`Epoch`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid inclusion proof for epoch root hash {0:?}")]
pub struct VerifyError(crate::proof::VerifyError<Epoch>);

impl VerifyError {
    /// Get the root hash against which the proof failed to verify.
    pub fn root(&self) -> Root {
        Root(self.0.root())
    }

    /// Extract the original proof from this error.
    pub fn into_proof(self) -> Proof {
        Proof(self.0.into_proof())
    }
}
