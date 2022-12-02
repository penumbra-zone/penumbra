pub mod connection_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenInitExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenInit) {
            let connection_id = ConnectionId::new(self.get_connection_counter().await.unwrap().0);

            let compatible_versions = vec![Version::default()];

            let new_connection_end = ConnectionEnd::new(
                ConnectionState::Init,
                msg.client_id_on_a.clone(),
                msg.counterparty.clone(),
                compatible_versions,
                msg.delay_period,
            );

            // commit the connection, this also increments the connection counter
            self.put_new_connection(&connection_id, new_connection_end)
                .await
                .unwrap();

            self.record(event::connection_open_init(
                &connection_id,
                &msg.client_id_on_a,
                &msg.counterparty,
            ));
        }
    }

    impl<T: StateWriteExt> ConnectionOpenInitExecute for T {}
}

pub mod connection_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenTryExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenTry) {
            // new_conn is the new connection that we will open on this chain
            let mut new_conn = ConnectionEnd::new(
                ConnectionState::TryOpen,
                msg.client_id_on_b.clone(),
                msg.counterparty.clone(),
                msg.versions_on_a.clone(),
                msg.delay_period,
            );
            new_conn.set_version(
                pick_version(SUPPORTED_VERSIONS.to_vec(), msg.versions_on_a.clone()).unwrap(),
            );

            let new_connection_id =
                ConnectionId::new(self.get_connection_counter().await.unwrap().0);

            // TODO(erwan): deprecated now?
            // if let Some(prev_conn_id) = &msg.previous_connection_id {
            //     // prev conn ID already validated in check_tx_stateful
            //     new_connection_id = prev_conn_id.clone();
            // }

            self.put_new_connection(&new_connection_id, new_conn)
                .await
                .unwrap();

            self.record(event::connection_open_try(
                &new_connection_id,
                &msg.client_id_on_b,
                &msg.counterparty,
            ));
        }
    }

    impl<T: StateWriteExt> ConnectionOpenTryExecute for T {}
}

pub mod connection_open_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenConfirmExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenConfirm) {
            let mut connection = self
                .get_connection(&msg.conn_id_on_b)
                .await
                .unwrap()
                .ok_or_else(|| anyhow::anyhow!("no connection with the given ID"))
                .unwrap();

            connection.set_state(ConnectionState::Open);

            self.update_connection(&msg.conn_id_on_b, connection.clone());

            self.record(event::connection_open_confirm(
                &msg.conn_id_on_b,
                &connection,
            ));
        }
    }

    impl<T: StateWriteExt> ConnectionOpenConfirmExecute for T {}
}
pub mod connection_open_ack {
    use super::super::*;

    #[async_trait]
    pub trait ConnectionOpenAckExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgConnectionOpenAck) {
            let mut connection = self
                .get_connection(&msg.conn_id_on_a)
                .await
                .unwrap()
                .unwrap();

            // TODO(erwan): reviewer should check that CP is correct pls
            let prev_counterparty = connection.counterparty();
            let counterparty = Counterparty::new(
                prev_counterparty.client_id().clone(),
                Some(msg.conn_id_on_b.clone()),
                prev_counterparty.prefix().clone(),
            );
            connection.set_state(ConnectionState::Open);
            connection.set_version(msg.version.clone());
            connection.set_counterparty(counterparty);

            self.update_connection(&msg.conn_id_on_a, connection.clone());

            self.record(event::connection_open_ack(&msg.conn_id_on_a, &connection));
        }
    }

    impl<T: StateWriteExt> ConnectionOpenAckExecute for T {}
}
