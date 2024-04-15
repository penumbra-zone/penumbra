use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_proto::StateWriteProto as _;

// #[async_trait]
// /// Debits an opened position NFT and credits a closed position NFT.
// impl ActionHandler for () {
//     type CheckStatelessContext = ();
//     async fn check_stateless(&self, _context: ()) -> Result<()> {
//         // Nothing to do: the only validation is of the state change,
//         // and that's done by the value balance mechanism.
//         Ok(())
//     }
//
//     async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
//         // state.record_proto(event::position_close(self));
//
//         Ok(())
//     }
// }
