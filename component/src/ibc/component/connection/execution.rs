pub mod connection_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenInitExecute: StateReadExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenInit) {
            let connection_id = ConnectionId::new(self.get_connection_counter().await.unwrap().0);

            let compatible_versions = vec![Version::default()];

            let new_connection_end = ConnectionEnd::new(
                ConnectionState::Init,
                msg.client_id.clone(),
                msg.counterparty.clone(),
                compatible_versions,
                msg.delay_period,
            );

            // commit the connection, this also increments the connection counter
            self.put_new_connection(&connection_id, new_connection_end)
                .await
                .unwrap();

            state.record(event::connection_open_init(
                &connection_id,
                &msg.client_id,
                &msg.counterparty,
            ));
        }
    }

    impl<T: StateReadExt> ConnectionOpenInitExecute for T {}
}

pub mod connection_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenTryExecute: StateReadExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenTry) {
            // new_conn is the new connection that we will open on this chain
            let mut new_conn = ConnectionEnd::new(
                ConnectionState::TryOpen,
                msg.client_id.clone(),
                msg.counterparty.clone(),
                msg.counterparty_versions.clone(),
                msg.delay_period,
            );
            new_conn.set_version(
                pick_version(
                    SUPPORTED_VERSIONS.to_vec(),
                    msg.counterparty_versions.clone(),
                )
                .unwrap(),
            );

            let mut new_connection_id =
                ConnectionId::new(self.get_connection_counter().await.unwrap().0);

            if let Some(prev_conn_id) = &msg.previous_connection_id {
                // prev conn ID already validated in check_tx_stateful
                new_connection_id = prev_conn_id.clone();
            }

            self.put_new_connection(&new_connection_id, new_conn)
                .await
                .unwrap();

            state.record(event::connection_open_try(
                &new_connection_id,
                &msg.client_id,
                &msg.counterparty,
            ));
        }
    }

    impl<T: StateReadExt> ConnectionOpenTryExecute for T {}
}

pub mod connection_open_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenConfirmExecute: StateReadExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenConfirm) {
            let mut connection = self
                .get_connection(&msg.connection_id)
                .await
                .unwrap()
                .ok_or_else(|| anyhow::anyhow!("no connection with the given ID"))
                .unwrap();

            connection.set_state(ConnectionState::Open);

            self.update_connection(&msg.connection_id, connection.clone())
                .await;

            state.record(event::connection_open_confirm(
                &msg.connection_id,
                &connection,
            ));
        }
    }

    impl<T: StateReadExt> ConnectionOpenConfirmExecute for T {}
}
pub mod connection_open_ack {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenAckExecute: StateReadExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenAck) {
            let mut connection = self
                .get_connection(&msg.connection_id)
                .await
                .unwrap()
                .unwrap();

            let prev_counterparty = connection.counterparty();
            let counterparty = Counterparty::new(
                prev_counterparty.client_id().clone(),
                Some(msg.counterparty_connection_id.clone()),
                prev_counterparty.prefix().clone(),
            );
            connection.set_state(ConnectionState::Open);
            connection.set_version(msg.version.clone());
            connection.set_counterparty(counterparty);

            self.update_connection(&msg.connection_id, connection.clone())
                .await;

            state.record(event::connection_open_ack(&msg.connection_id, &connection));
        }
    }

    impl<T: StateReadExt> ConnectionOpenAckExecute for T {}
}
