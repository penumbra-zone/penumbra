#[macro_use]
extern crate proptest_derive;

use std::collections::HashSet;

use proptest::{arbitrary::*, prelude::*};

use penumbra_tct::{validate, StateCommitment, Tree, Witness};

const MAX_USED_COMMITMENTS: usize = 3;
const MAX_TIER_ACTIONS: usize = 10;

#[derive(Debug, Copy, Clone, Arbitrary)]
#[proptest(params("Vec<StateCommitment>"))]
enum Action {
    EndBlock,
    EndEpoch,
    Forget(StateCommitment),
    Insert(Witness, StateCommitment),
}

impl Action {
    fn apply(&self, tree: &mut Tree) -> anyhow::Result<()> {
        // Predict the position of the next insertion
        let predicted_position = tree.position();

        match self {
            Action::Insert(witness, commitment) => {
                // Insert the commitment
                tree.insert(*witness, *commitment)?;

                // If the insertion succeeded, the position must have been non-`None`
                assert!(predicted_position.is_some());

                // If the commitment was witnessed, the position must match the position of the
                // commitment when retrieved, and the proof must validate and contain the correct
                // commitment
                if matches!(witness, Witness::Keep) {
                    let commitment_position = tree.position_of(*commitment);
                    assert!(commitment_position.is_some());
                    assert_eq!(predicted_position, commitment_position);

                    let proof = tree.witness(*commitment).unwrap();
                    assert_eq!(*commitment, proof.commitment());

                    assert!(proof.verify(tree.root()).is_ok());
                }

                // Check that the position advanced by one commitment
                let old_position = predicted_position.unwrap();
                let new_position = tree.position().unwrap();

                assert_eq!(new_position.epoch(), old_position.epoch());
                assert_eq!(new_position.block(), old_position.block());
                assert_eq!(new_position.commitment(), old_position.commitment() + 1);
            }
            Action::EndBlock => {
                tree.end_block()?;

                let old_position = predicted_position.unwrap();
                let new_position = tree.position().unwrap();

                assert_eq!(new_position.epoch(), old_position.epoch());
                assert_eq!(new_position.block(), old_position.block() + 1);
                assert_eq!(new_position.commitment(), 0);
            }
            Action::EndEpoch => {
                tree.end_epoch()?;

                let old_position = predicted_position.unwrap();
                let new_position = tree.position().unwrap();

                assert_eq!(new_position.epoch(), old_position.epoch() + 1);
                assert_eq!(new_position.block(), 0);
                assert_eq!(new_position.commitment(), 0);
            }
            Action::Forget(commitment) => {
                let exists = tree.witness(*commitment).is_some();
                let result = tree.forget(*commitment);
                assert_eq!(exists, result);
            }
        };

        Ok(())
    }
}

proptest! {
    #[test]
    fn index_correct(
        actions in
            prop::collection::vec(any::<StateCommitment>(), 1..MAX_USED_COMMITMENTS)
                .prop_flat_map(|commitments| {
                    prop::collection::vec(any_with::<Action>(commitments), 1..MAX_TIER_ACTIONS)
                })
    ) {
        let mut tree = Tree::new();

        let mut commitments_added = HashSet::new();

        for action in &actions {
            match action {
                Action::Insert (Witness::Keep, commitment) => {
                    commitments_added.insert(commitment);
                },
                Action::Forget (commitment) => {
                    commitments_added.remove(&commitment);
                },
                _ => {}
            }
            action.apply(&mut tree).unwrap();
        }

        // Check generated commitments
        for commitment in commitments_added {
            let commitment_position = tree.position_of(*commitment);
            assert!(commitment_position.is_some());

            let proof = tree.witness(*commitment).unwrap();
            assert_eq!(*commitment, proof.commitment());

            assert!(proof.verify(tree.root()).is_ok());
        }
    }

    #[test]
    fn validate_index(
        actions in
            prop::collection::vec(any::<StateCommitment>(), 1..MAX_USED_COMMITMENTS)
                .prop_flat_map(|commitments| {
                    prop::collection::vec(any_with::<Action>(commitments), 1..MAX_TIER_ACTIONS)
                })
    ) {
        let mut tree = Tree::new();
        for action in actions {
            action.apply(&mut tree).unwrap();
        }
        validate::index(&tree).unwrap();
    }

    #[test]
    fn verify_all_proofs(
        actions in
            prop::collection::vec(any::<StateCommitment>(), 1..MAX_USED_COMMITMENTS)
                .prop_flat_map(|commitments| {
                    prop::collection::vec(any_with::<Action>(commitments), 1..MAX_TIER_ACTIONS)
                })
    ) {
        let mut tree = Tree::new();
        for action in actions {
            action.apply(&mut tree).unwrap();
        }
        validate::all_proofs(&tree).unwrap();
    }

    #[test]
    fn validate_cached_hashes(
        actions in
            prop::collection::vec(any::<StateCommitment>(), 1..MAX_USED_COMMITMENTS)
                .prop_flat_map(|commitments| {
                    prop::collection::vec(any_with::<Action>(commitments), 1..MAX_TIER_ACTIONS)
                })
    ) {
        let mut tree = Tree::new();
        for action in actions {
            action.apply(&mut tree).unwrap();
        }
        validate::cached_hashes(&tree).unwrap();
    }


    #[test]
    fn validate_forgotten(
        actions in
            prop::collection::vec(any::<StateCommitment>(), 1..MAX_USED_COMMITMENTS)
                .prop_flat_map(|commitments| {
                    prop::collection::vec(any_with::<Action>(commitments), 1..MAX_TIER_ACTIONS)
                })
    ) {
        let mut tree = Tree::new();
        for action in actions {
            // Number of commitments forgotten already
            let pre = tree.forgotten();

            // The number of forgotten commitments should increase if the commitment is contained
            // and the action is about to forget it: the common case in practice is `forget`, but if
            // the same commitment is inserted twice, both times with `Witness::Keep`, this will
            // also increment the count
            let should_increase = if let Action::Forget(commitment) | Action::Insert(Witness::Keep, commitment) = action {
                tree.position_of(commitment).is_some()
            } else {
                false
            };

            // Apply the action
            action.apply(&mut tree).unwrap();

            // Number of commitments forgotten after the action
            let post = tree.forgotten();

            // Check that the count is increasing correctly
            if should_increase {
                assert_eq!(post, pre.next());
            } else {
                assert_eq!(post, pre);
            }
        }
        validate::forgotten(&tree).unwrap();
    }
}
