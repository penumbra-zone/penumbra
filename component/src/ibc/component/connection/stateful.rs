pub mod connection_open_init {
    use crate::ibc::component::client::StateReadExt as _;

    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenInitCheck: StateReadExt {
        async fn validate(&self, msg: &MsgConnectionOpenInit) -> anyhow::Result<()> {
            // check that the client with the specified ID exists
            self.get_client_state(&msg.client_id_on_a).await?;
            self.get_client_type(&msg.client_id_on_a).await?;

            Ok(())
        }
    }

    impl<T: StateReadExt> ConnectionOpenInitCheck for T {}
}

pub mod connection_open_confirm {
    use crate::ibc::component::client::StateReadExt as _;

    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenConfirmCheck: StateReadExt + inner::Inner {
        // Validate a ConnectionOpenConfirm message, completing the IBC connection handshake.
        //
        // Verify that we have a connection in the correct state (TRYOPEN), and that the
        // counterparty has committed a connection with the expected state (OPEN) to their state
        // store.
        //
        // Here we are Chain B.
        // CHAINS:          (A, B)
        // PRIOR STATE:     (OPEN, TRYOPEN)
        // POSTERIOR STATE: (OPEN, OPEN)
        async fn validate(&self, msg: &MsgConnectionOpenConfirm) -> anyhow::Result<()> {
            // verify that a connection with the provided ID exists and is in the correct state
            // (TRYOPEN)
            let connection = self.verify_previous_connection(msg).await?;

            let expected_conn = ConnectionEnd::new(
                ConnectionState::Open,
                connection.counterparty().client_id().clone(),
                Counterparty::new(
                    connection.client_id().clone(),
                    Some(msg.conn_id_on_b.clone()),
                    penumbra_chain::PENUMBRA_COMMITMENT_PREFIX.clone(),
                ),
                connection.versions().to_vec(),
                connection.delay_period(),
            );

            // get the trusted client state for the counterparty
            let trusted_client_state = self.get_client_state(connection.client_id()).await?;

            // check if the client is frozen
            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            // get the stored consensus state for the counterparty
            let trusted_consensus_state = self
                .get_verified_consensus_state(msg.proof_height_on_a, connection.client_id().clone())
                .await?;

            // PROOF VERIFICATION
            // in connectionOpenConfirm, only the inclusion of the connection state must be
            // verified, not the client or consensus states.
            trusted_client_state.verify_connection_state(
                msg.proof_height_on_a,
                connection.counterparty().prefix(),
                &msg.proof_conn_end_on_a,
                trusted_consensus_state.root(),
                connection
                    .counterparty()
                    .connection_id()
                    .ok_or_else(|| anyhow::anyhow!("invalid counterparty"))?,
                &expected_conn,
            )?;

            Ok(())
        }
    }
    mod inner {
        use super::*;

        #[async_trait]
        pub trait Inner: StateReadExt {
            async fn verify_previous_connection(
                &self,
                msg: &MsgConnectionOpenConfirm,
            ) -> anyhow::Result<ConnectionEnd> {
                let connection = self
                    .get_connection(&msg.conn_id_on_b)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("connection not found"))?;

                if !connection.state_matches(&ConnectionState::TryOpen) {
                    return Err(anyhow::anyhow!("connection not in correct state"));
                } else {
                    Ok(connection)
                }
            }
        }
        impl<T: StateReadExt> Inner for T {}
    }
    impl<T: StateReadExt> ConnectionOpenConfirmCheck for T {}
}

pub mod connection_open_ack {
    use crate::ibc::component::client::StateReadExt as _;

    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenAckCheck: StateReadExt + inner::Inner {
        // Validate a ConnectionOpenAck message, which is sent to us by a counterparty chain that
        // has committed a Connection to us expected to be in the TRYOPEN state. Before executing a
        // ConnectionOpenAck, we must have a prior connection to this chain in the INIT state.
        //
        // In order to verify a ConnectionOpenAck, we need to check that the counterparty chain has
        // committed a _valid_ Penumbra consensus state, that the counterparty chain has committed
        // the expected Connection to its state (in the TRYOPEN state) with the expected version,
        // and that the counterparty has committed a correct Penumbra client state to its state.
        //
        // Here we are Chain A.
        // CHAINS:          (A, B)
        // PRIOR STATE:     (INIT, TRYOPEN)
        // POSTERIOR STATE: (OPEN, TRYOPEN)
        async fn validate(&self, msg: &MsgConnectionOpenAck) -> anyhow::Result<()> {
            // verify that the consensus height is correct
            self.consensus_height_is_correct(msg).await?;

            // verify that the client state is well formed
            self.penumbra_client_state_is_well_formed(msg).await?;

            // verify the previous connection that we're ACKing is in the correct state
            let connection = self.verify_previous_connection(msg).await?;

            // verify that the counterparty committed a TRYOPEN connection with us as the
            // counterparty
            let expected_counterparty = Counterparty::new(
                connection.client_id().clone(), // client ID (local)
                Some(msg.conn_id_on_a.clone()), // connection ID (local)
                penumbra_chain::PENUMBRA_COMMITMENT_PREFIX.clone(), // commitment prefix (local)
            );

            // the connection we expect the counterparty to have committed
            let expected_conn = ConnectionEnd::new(
                ConnectionState::TryOpen,
                connection.counterparty().client_id().clone(),
                expected_counterparty.clone(),
                vec![msg.version.clone()],
                connection.delay_period(),
            );

            // get the stored client state for the counterparty
            let trusted_client_state = self.get_client_state(connection.client_id()).await?;

            // check if the client is frozen
            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            // get the stored consensus state for the counterparty
            let trusted_consensus_state = self
                .get_verified_consensus_state(
                    msg.proofs_height_on_b,
                    connection.client_id().clone(),
                )
                .await?;

            // PROOF VERIFICATION
            // 1. verify that the counterparty chain committed the expected_conn to its state
            trusted_client_state
                .verify_connection_state(
                    msg.proofs_height_on_b,
                    connection.counterparty().prefix(),
                    &msg.proof_conn_end_on_b,
                    trusted_consensus_state.root(),
                    &msg.conn_id_on_b,
                    &expected_conn,
                )
                .map_err(|e| anyhow::anyhow!("couldn't verify connection state: {}", e))?;

            // 2. verify that the counterparty chain committed the correct ClientState (that was
            //    provided in the msg)
            trusted_client_state
                .verify_client_full_state(
                    msg.proofs_height_on_b,
                    connection.counterparty().prefix(),
                    &msg.proof_client_state_of_a_on_b,
                    trusted_consensus_state.root(),
                    connection.counterparty().client_id(),
                    msg.client_state_of_a_on_b.clone(),
                )
                .map_err(|e| anyhow::anyhow!("couldn't verify client state: {}", e))?;

            let expected_consensus = self
                .get_penumbra_consensus_state(msg.consensus_height_of_a_on_b)
                .await?;

            // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
            //    the given consensus height
            trusted_client_state
                .verify_client_consensus_state(
                    msg.proofs_height_on_b,
                    connection.counterparty().prefix(),
                    &msg.proof_consensus_state_of_a_on_b,
                    trusted_consensus_state.root(),
                    connection.counterparty().client_id(),
                    msg.consensus_height_of_a_on_b,
                    &expected_consensus,
                )
                .map_err(|e| anyhow::anyhow!("couldn't verify client consensus state: {}", e))?;

            Ok(())
        }
    }
    mod inner {
        use penumbra_chain::StateReadExt as _;

        use super::*;

        #[async_trait]
        pub trait Inner: StateReadExt {
            async fn consensus_height_is_correct(
                &self,
                msg: &MsgConnectionOpenAck,
            ) -> anyhow::Result<()> {
                let current_height = IBCHeight::new(0, self.get_block_height().await?)?;
                if msg.consensus_height_of_a_on_b > current_height {
                    return Err(anyhow::anyhow!(
                        "consensus height is greater than the current block height",
                    ));
                }

                Ok(())
            }
            async fn penumbra_client_state_is_well_formed(
                &self,
                msg: &MsgConnectionOpenAck,
            ) -> anyhow::Result<()> {
                let height = self.get_block_height().await?;
                let chain_id = self.get_chain_id().await?;
                _ = validate_penumbra_client_state(
                    msg.client_state_of_a_on_b.clone(),
                    &chain_id,
                    height,
                )?;

                Ok(())
            }
            async fn verify_previous_connection(
                &self,
                msg: &MsgConnectionOpenAck,
            ) -> anyhow::Result<ConnectionEnd> {
                let connection = self
                    .get_connection(&msg.conn_id_on_b)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("no connection with the specified ID exists"))?;

                // see
                // https://github.com/cosmos/ibc/blob/master/spec/core/ics-003-connection-semantics/README.md
                // for this validation logic
                let state_is_consistent = connection.state_matches(&ConnectionState::Init)
                    && connection.versions().contains(&msg.version)
                    || connection.state_matches(&ConnectionState::TryOpen)
                        && connection.versions().get(0).eq(&Some(&msg.version));

                if !state_is_consistent {
                    return Err(anyhow::anyhow!("connection is not in the correct state"));
                } else {
                    Ok(connection)
                }
            }
        }
        impl<T: StateReadExt> Inner for T {}
    }
    impl<T: StateReadExt> ConnectionOpenAckCheck for T {}
}

pub mod connection_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenTryCheck:
        inner::Inner + crate::ibc::component::client::StateReadExt
    {
        // Validate a ConnectionOpenTry message, which is sent to us by a counterparty chain that
        // has committed a Connection to us in an INIT state on its chain. Before executing a
        // ConnectionOpenTry message, we have no knowledge about the connection: our counterparty
        // is in INIT state, and we are in none state. After executing ConnectionOpenTry, our
        // counterparty is in INIT state, and we are in TRYOPEN state.
        //
        // In order to verify a ConnectionOpenTry, we need to check that the counterparty chain has
        // committed a _valid_ Penumbra consensus state, that the counterparty chain has committed
        // the expected Connection to its state (in the INIT state), and that the counterparty has
        // committed a correct Penumbra client state to its state.
        //
        // Here we are Chain B.
        // CHAINS:          (A, B)
        // PRIOR STATE:     (INIT, none)
        // POSTERIOR STATE: (INIT, TRYOPEN)
        async fn validate(&self, msg: &MsgConnectionOpenTry) -> anyhow::Result<()> {
            // verify that the consensus height is correct
            self.consensus_height_is_correct(msg).await?;

            // verify that the client state (which is a Penumbra client) is well-formed for a
            // penumbra client.
            self.penumbra_client_state_is_well_formed(msg).await?;

            // TODO(erwan): how to handle this with ibc-rs@0.23.0?
            // if this msg provides a previous_connection_id to resume from, then check that the
            // provided previous connection ID is valid
            // let previous_connection = self.check_previous_connection(msg).await?;

            // perform version intersection
            // let supported_versions = previous_connection
            //     .map(|c| c.versions().to_vec())
            //     .unwrap_or_else(|| SUPPORTED_VERSIONS.clone());
            let supported_versions = SUPPORTED_VERSIONS.clone();

            pick_version(supported_versions, msg.versions_on_a.clone())?;

            // expected_conn is the conn that we expect to have been committed to on the counterparty
            // chain
            let expected_conn = ConnectionEnd::new(
                ConnectionState::Init,
                msg.counterparty.client_id().clone(),
                Counterparty::new(
                    msg.counterparty.client_id().clone(),
                    None,
                    penumbra_chain::PENUMBRA_COMMITMENT_PREFIX.clone(),
                ),
                msg.versions_on_a.clone(),
                msg.delay_period,
            );

            // get the stored client state for the counterparty
            let trusted_client_state = self.get_client_state(msg.counterparty.client_id()).await?;

            // check if the client is frozen
            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            // get the stored consensus state for the counterparty
            let trusted_consensus_state = self
                .get_verified_consensus_state(
                    msg.proofs_height_on_a,
                    msg.counterparty.client_id().clone(),
                )
                .await?;

            // PROOF VERIFICATION
            // 1. verify that the counterparty chain committed the expected_conn to its state
            trusted_client_state.verify_connection_state(
                msg.proofs_height_on_a,
                msg.counterparty.prefix(),
                &msg.proof_conn_end_on_a,
                trusted_consensus_state.root(),
                msg.counterparty
                    .connection_id
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("counterparty connection id is not set"))?,
                &expected_conn,
            )?;

            // 2. verify that the counterparty chain committed the correct ClientState (that was
            //    provided in the msg)
            trusted_client_state.verify_client_full_state(
                msg.proofs_height_on_a,
                msg.counterparty.prefix(),
                &msg.proof_client_state_of_b_on_a,
                trusted_consensus_state.root(),
                msg.counterparty.client_id(),
                msg.client_state_of_b_on_a.clone(),
            )?;

            let expected_consensus = self
                .get_penumbra_consensus_state(msg.consensus_height_of_b_on_a)
                .await?;

            // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
            //    the given consensus height
            trusted_client_state.verify_client_consensus_state(
                msg.proofs_height_on_a,
                msg.counterparty.prefix(),
                &msg.proof_consensus_state_of_b_on_a,
                trusted_consensus_state.root(),
                msg.counterparty.client_id(),
                msg.consensus_height_of_b_on_a,
                &expected_consensus,
            )?;

            Ok(())
        }
    }
    mod inner {

        use penumbra_chain::StateReadExt as _;

        use super::*;

        #[async_trait]
        pub trait Inner: StateReadExt {
            async fn consensus_height_is_correct(
                &self,
                msg: &MsgConnectionOpenTry,
            ) -> anyhow::Result<()> {
                let current_height = IBCHeight::new(0, self.get_block_height().await?)?;
                if msg.consensus_height_of_b_on_a > current_height {
                    return Err(anyhow::anyhow!(
                        "consensus height is greater than the current block height",
                    ));
                }

                Ok(())
            }
            async fn penumbra_client_state_is_well_formed(
                &self,
                msg: &MsgConnectionOpenTry,
            ) -> anyhow::Result<()> {
                let height = self.get_block_height().await?;
                let chain_id = self.get_chain_id().await?;
                validate_penumbra_client_state(
                    msg.client_state_of_b_on_a.clone(),
                    &chain_id,
                    height,
                )?;

                Ok(())
            }

            // TODO(erwan): how to handle this with ibc-rs@0.23.0?
            //async fn check_previous_connection(
            //    &self,
            //    msg: &MsgConnectionOpenTry,
            //) -> anyhow::Result<Option<ConnectionEnd>> {
            //    if let Some(prev_conn_id) = &msg.previous_connection_id {
            //        // check that we have a valid connection with the given ID
            //        let prev_connection = self
            //            .get_connection(prev_conn_id)
            //            .await?
            //            .ok_or_else(|| anyhow::anyhow!("no connection with the given ID"))?;

            //        // check that the existing connection matches the incoming connectionOpenTry
            //        if !(prev_connection.state_matches(&ConnectionState::Init)
            //            && prev_connection.counterparty_matches(&msg.counterparty)
            //            && prev_connection.client_id_matches(&msg.client_id)
            //            && prev_connection.delay_period() == msg.delay_period)
            //        {
            //            return Err(anyhow::anyhow!(
            //                "connection with the given ID is not in the correct state",
            //            ));
            //        }
            //        return Ok(Some(prev_connection));
            //    } else {
            //        return Ok(None);
            //    }
            //}
        }

        impl<T: StateReadExt> Inner for T {}
    }

    impl<T: StateReadExt> ConnectionOpenTryCheck for T {}
}
