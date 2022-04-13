use proptest::prelude::*;

use super::*;

use real::arbitrary::CommitmentStrategy;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Params"))]
pub enum Action {
    Insert(
        Witness,
        #[proptest(strategy = "CommitmentStrategy::one_of(params.commitments.clone())")] Commitment,
    ),
    Forget(
        #[proptest(strategy = "CommitmentStrategy::one_of(params.commitments.clone())")] Commitment,
    ),
    InsertBlock(
        #[proptest(
            strategy = "prop::collection::vec(any_with::<block::Action>(params.commitments), 0..params.max_tier_actions)"
        )]
        Vec<block::Action>,
    ),
    InsertBlockRoot(real::block::Root),
}

impl Simulate for Action {
    type Spec = spec::epoch::Builder;
    type Real = real::Epoch;

    fn simulate(self, spec: &mut Self::Spec, real: &mut Self::Real) {
        match self {
            Action::Insert(witness, commitment) => assert_eq!(
                spec.insert(witness, commitment),
                real.insert(witness, commitment).map_err(Into::into),
                "result mismatch from `Epoch::insert`"
            ),
            Action::Forget(commitment) => {
                assert_eq!(
                    spec.forget(commitment),
                    real.forget(commitment),
                    "result mismatch from `Epoch::forget`"
                )
            }
            Action::InsertBlock(actions) => {
                let mut spec_block = spec::block::Builder::default();
                let mut real_block = real::Block::default();
                actions.simulate(&mut spec_block, &mut real_block);
                assert_eq!(
                    spec.insert_block(spec_block),
                    real.insert_block(real_block).map_err(Into::into),
                    "result mismatch from `Epoch::insert_block`"
                );
            }
            Action::InsertBlockRoot(root) => {
                assert_eq!(
                    spec.insert_block_root(root),
                    real.insert_block_root(root).map_err(Into::into),
                    "result mismatch from `Epoch::insert_block_root`"
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
pub enum Observation {
    Witness(#[proptest(strategy = "CommitmentStrategy::one_of(params)")] Commitment),
    Root,
    CurrentBlockRoot,
    Position,
    WitnessedCount,
    IsEmpty,
}

impl Simulate for Observation {
    type Spec = spec::Epoch;
    type Real = real::Epoch;

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
                    "result mismatch from `Epoch::witness`"
                );
                // If we got this far, any proof will do: check that it verifies against the real
                // and spec roots (which should be the same but we check both just in case)
                if let Some(proof) = real_proof {
                    assert!(
                        proof.verify(real.root()).is_ok(),
                        "proof failed to verify for implementation after `Epoch::witness`"
                    );
                    assert!(
                        proof.verify(spec.root()).is_ok(),
                        "proof failed to verify for specification after `Epoch::witness`"
                    );
                }
            }
            Root => assert_eq!(
                spec.root(),
                real.root(),
                "result mismatch from `Epoch::root`"
            ),
            CurrentBlockRoot => assert_eq!(
                spec.current_block_root(),
                real.current_block_root(),
                "result mismatch from `Epoch::current_block_root`"
            ),
            Position => assert_eq!(
                spec.position(),
                real.position(),
                "result mismatch from `Epoch::position`"
            ),
            WitnessedCount => assert_eq!(
                spec.witnessed_count(),
                real.witnessed_count(),
                "result mismatch from `Epoch::witnessed_count`"
            ),
            IsEmpty => assert_eq!(
                spec.is_empty(),
                real.is_empty(),
                "result mismatch from `Epoch::is_empty`"
            ),
        }
    }
}
