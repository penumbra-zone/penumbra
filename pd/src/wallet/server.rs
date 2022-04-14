use jmt::WriteOverlay;

mod light;
mod thin;

struct WalletOverlay<T>(WriteOverlay<T>);

impl<T> WalletOverlay<T> {
    /// Checks a provided chain_id against the chain state.
    ///
    /// Passes through if the provided chain_id is empty or matches, and
    /// otherwise errors.
    fn check_chain_id(&self, provided: &str) -> Result<(), tonic::Status> {
        if provided.is_empty() || self.chain_params_rx().borrow().chain_id == provided {
            Ok(())
        } else {
            Err(tonic::Status::failed_precondition(format!(
                "provided chain_id {} does not match chain_id {}",
                provided,
                self.0.chain_params_rx().borrow().chain_id
            )))
        }
    }
}
