#[macro_use]
extern crate proptest_derive;

use std::fmt::{Debug, Display};

use proptest::{arbitrary::*, prelude::*};

use penumbra_tct::{storage::InMemory, validate, Commitment, Tree, Witness};

const MAX_USED_COMMITMENTS: usize = 3;
const MAX_TIER_ACTIONS: usize = 10;

#[derive(Debug, Copy, Clone, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
enum Action {
    Serialize,
    EvaluateRoot,
    EndEpoch,
    EndBlock,
    Insert(Witness, Commitment),
    Forget(Commitment),
}

impl Action {
    fn apply(&self, state: &mut InMemory, tree: &mut Tree) -> anyhow::Result<()> {
        match self {
            Action::Insert(witness, commitment) => {
                tree.insert(*witness, *commitment)?;
            }
            Action::EndBlock => {
                tree.end_block()?;
            }
            Action::EndEpoch => {
                tree.end_epoch()?;
            }
            Action::EvaluateRoot => {
                let _ = tree.root();
            }
            Action::Forget(commitment) => {
                tree.forget(*commitment);
            }
            Action::Serialize => {
                tree.to_writer(state)?;
            }
        };

        Ok(())
    }
}

proptest! {
    #[test]
    fn incremental_serialize(
        sparse in any::<bool>(),
        actions in
            prop::collection::vec(any::<Commitment>(), 1..MAX_USED_COMMITMENTS)
                .prop_flat_map(|commitments| {
                    prop::collection::vec(any_with::<Action>(commitments), 1..MAX_TIER_ACTIONS)
                })
                .prop_map(|mut actions| {
                    // Ensure that every sequence of actions ends in a serialization
                    actions.push(Action::Serialize);
                    actions
                })
    ) {
            let mut tree = Tree::new();
            let mut incremental = if sparse {
                InMemory::new_sparse()
            } else {
                InMemory::new()
            };

            // Run all the actions in sequence
            for action in actions {
                action.apply(&mut incremental, &mut tree).unwrap();
            }

            // Make a new copy of the tree by deserializing from the storage
            let deserialized = Tree::from_reader(&mut incremental).unwrap();

           // After running all the actions, the deserialization of the stored tree should match
            // our in-memory tree (this only holds because we ensured that the last action is always
            // a `Serialize`)
            assert_eq!(tree, deserialized, "mismatch when deserializing from storage: {incremental:?}");

            // It should also hold that the result of any sequence of incremental serialization is
            // the same as merely serializing the result all at once, after the fact
            let mut non_incremental = if sparse {
                InMemory::new_sparse()
            } else {
                InMemory::new()
            };

            // To check this, we first serialize to a new in-memory storage instance
            tree.to_writer(&mut non_incremental).unwrap();

            // Then we check both that the storage matches the incrementally-built one
            assert_eq!(incremental, non_incremental, "incremental storage mismatches non-incremental storage");

            // Higher-order helper function to factor out common behavior of validation assertions
            #[allow(clippy::type_complexity)]
            fn v<E: Display + Debug + 'static>(validate: fn(&Tree) -> Result<(), E>) -> Box<dyn Fn(&Tree, &Tree, &InMemory)> {
                Box::new(move |original, deserialized, storage| if let Err(error) = validate(deserialized) {
                    panic!("{error}:\n\nERROR: {error:?}\n\nORIGINAL: {original:?}\n\nDESERIALIZED: {deserialized:?}\n\nSTORAGE: {storage:?}");
                })
            }

             // Validate the internal structure of the deserialized tree
            for validate in [
                v(validate::index),
                v(validate::all_proofs),
                v(validate::cached_hashes),
                v(validate::forgotten)
            ] {
                validate(&tree, &deserialized, &incremental);
            }
    }
}
