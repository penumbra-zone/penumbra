mod light;
mod thin;

// TODO: implement for WriteOverlay
// impl state::Reader {
//     /// Checks a provided chain_id against the chain state.
//     ///
//     /// Passes through if the provided chain_id is empty or matches, and
//     /// otherwise errors.
//     fn check_chain_id(&self, provided: &str) -> Result<(), tonic::Status> {
//         if provided.is_empty() || self.chain_params_rx().borrow().chain_id == provided {
//             Ok(())
//         } else {
//             Err(tonic::Status::failed_precondition(format!(
//                 "provided chain_id {} does not match chain_id {}",
//                 provided,
//                 self.chain_params_rx().borrow().chain_id
//             )))
//         }
//     }
// }
