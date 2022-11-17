#[async_trait]
impl ActionHandler for IbcAction {
    #[instrument(name = "ics2_client", skip(tx))]
    fn check_stateless(&self, context: Arc<Transaction>) -> Result<()> {
        // Each stateless check is a distinct function in an appropriate submodule,
        // so that we can easily add new stateless checks and see a birds' eye view
        // of all of the checks we're performing.

        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(CreateClient(msg)) => {
                    use stateless::create_client::*;
                    let msg = MsgCreateAnyClient::try_from(msg.clone())?;

                    client_state_is_tendermint(&msg)?;
                    consensus_state_is_tendermint(&msg)?;
                }
                Some(UpdateClient(msg)) => {
                    use stateless::update_client::*;
                    let msg = MsgUpdateAnyClient::try_from(msg.clone())?;

                    header_is_tendermint(&msg)?;
                }
                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ics2_client", skip(state, tx))]
    async fn check_stateful(&self, state: Arc<State>, context: Arc<Transaction>) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(CreateClient(msg)) => {
                    use stateful::create_client::CreateClientCheck;
                    let msg = MsgCreateAnyClient::try_from(msg.clone())?;
                    state.validate(&msg).await?;
                }
                Some(UpdateClient(msg)) => {
                    use stateful::update_client::UpdateClientCheck;
                    let msg = MsgUpdateAnyClient::try_from(msg.clone())?;
                    state.validate(&msg).await?;
                }
                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }
        Ok(())
    }

    #[instrument(name = "ics2_client", skip(state, tx))]
    async fn execute(&self, state: &mut StateTransaction) -> Result<()> {
        // Handle any IBC actions found in the transaction.
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(CreateClient(raw_msg_create_client)) => {
                    let msg_create_client =
                        MsgCreateAnyClient::try_from(raw_msg_create_client.clone()).unwrap();

                    state.execute_create_client(msg_create_client).await;
                }
                Some(UpdateClient(raw_msg_update_client)) => {
                    let msg_update_client =
                        MsgUpdateAnyClient::try_from(raw_msg_update_client.clone()).unwrap();

                    state.execute_update_client(msg_update_client).await;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
