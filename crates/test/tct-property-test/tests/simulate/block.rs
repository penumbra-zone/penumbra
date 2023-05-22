use real::arbitrary::CommitmentStrategy;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Params"))]
pub enum Action {
    ForceRoot,
    Insert(
        Witness,
        #[proptest(strategy = "CommitmentStrategy::one_of(params.commitments.clone())")] Commitment,
    ),
    Forget(#[proptest(strategy = "CommitmentStrategy::one_of(params.commitments)")] Commitment),
}

impl Simulate for Action {
    type Spec = spec::block::Builder;
    type Real = real::Block;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        match self {
            Action::ForceRoot => {
                // There is no equivalent action to forcing the root of the specification, because
                // the root is not known when it is a `Builder`.
                real.root();
            }
            Action::Insert(witness, commitment) => assert_eq!(
                spec.insert(witness, commitment),
                real.insert(witness, commitment).map_err(Into::into),
                "result mismatch from `Block::insert`"
            ),
            Action::Forget(commitment) => {
                assert_eq!(
                    spec.forget(commitment),
                    real.forget(commitment),
                    "result mismatch from `Block::forget`"
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
pub enum Observation {
    Witness(#[proptest(strategy = "CommitmentStrategy::one_of(params.clone())")] Commitment),
    Root,
    Position,
    PositionOf(#[proptest(strategy = "CommitmentStrategy::one_of(params)")] Commitment),
    WitnessedCount,
    IsEmpty,
}

impl Simulate for Observation {
    type Spec = spec::Block;
    type Real = real::Block;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        use Observation::*;
        match self {
            Witness(commitment) => {
                // Get a proof from the spec and the real implementation
                let spec_proof = spec.witness(commitment);
                let real_proof = real.witness(commitment);
                // Assert that they are identical (or that they are both None)
                assert_eq!(
                    spec_proof, real_proof,
                    "result mismatch from `Block::witness`"
                );
                // If we got this far, any proof will do: check that it verifies against the real
                // and spec roots (which should be the same but we check both just in case)
                if let Some(proof) = real_proof {
                    assert!(
                        proof.verify(real.root()).is_ok(),
                        "proof verification failed for implementation after `Block::witness`"
                    );
                    assert!(
                        proof.verify(spec.root()).is_ok(),
                        "proof verification failed for specification after `Block::witness`"
                    );
                }
            }
            Root => assert_eq!(
                spec.root(),
                real.root(),
                "result mismatch from `Block::root`"
            ),
            Position => assert_eq!(
                spec.position(),
                real.position(),
                "result mismatch from `Block::position`"
            ),
            PositionOf(commitment) => assert_eq!(
                spec.position_of(commitment),
                real.position_of(commitment),
                "result mismatch from `Block::position_of`"
            ),
            WitnessedCount => assert_eq!(
                spec.witnessed_count(),
                real.witnessed_count(),
                "result mismatch from `Block::witnessed_count"
            ),
            IsEmpty => assert_eq!(
                spec.is_empty(),
                real.is_empty(),
                "result mismatch from `Block::is_empty"
            ),
        }
    }
}
