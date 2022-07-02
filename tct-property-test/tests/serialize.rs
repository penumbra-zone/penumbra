#[macro_use]
extern crate proptest_derive;

use std::collections::HashSet;

use proptest::{arbitrary::*, prelude::*};

use penumbra_tct::{
    storage::{deserialize, serialize, InMemory},
    Commitment, Forgotten, Tree, Witness,
};

const MAX_USED_COMMITMENTS: usize = 3;
const MAX_TIER_ACTIONS: usize = 10;

#[derive(Debug, Copy, Clone, Arbitrary)]
#[proptest(params("Vec<Commitment>"))]
enum Action {
    EndBlock,
    EndEpoch,
    Forget(Commitment),
    Insert(Witness, Commitment),
    Serialize,
}

struct State {
    last_forgotten: Forgotten,
    storage: InMemory,
}

impl Action {
    async fn apply(&self, state: &mut State, tree: &mut Tree) -> anyhow::Result<()> {
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
            Action::Forget(commitment) => {
                tree.forget(*commitment);
            }
            Action::Serialize => {
                serialize::to_writer(
                    serialize::Options::default(),
                    state.last_forgotten,
                    &mut state.storage,
                    tree,
                )
                .await?;

                state.last_forgotten = tree.forgotten();
            }
        };

        Ok(())
    }
}
