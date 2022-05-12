pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitCheck: StateExt + inner::Inner {
        async fn validate(&self, msg: &MsgChannelOpenInit) -> anyhow::Result<()> {
            let channel_id = self.get_channel_id().await?;

            self.verify_channel_does_not_exist(&channel_id, &msg.port_id)
                .await?;

            // NOTE: optimistic channel handshakes are allowed, so we don't check if the connection
            // is open here.
            self.verify_connections_exist(&msg).await?;

            // TODO: do we want to do capability authentication?

            Ok(())
        }
    }
    mod inner {
        use super::*;

        #[async_trait]
        pub trait Inner: StateExt {
            async fn verify_connections_exist(
                &self,
                msg: &MsgChannelOpenInit,
            ) -> anyhow::Result<()> {
                self.get_connection(&msg.channel.connection_hops[0])
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("connection not found"))
                    .map(|_| ())
            }
            async fn get_channel_id(&self) -> anyhow::Result<ChannelId> {
                let counter = self.get_channel_counter().await?;

                Ok(ChannelId::new(counter))
            }
            async fn verify_channel_does_not_exist(
                &self,
                channel_id: &ChannelId,
                port_id: &PortId,
            ) -> anyhow::Result<()> {
                let channel = self.get_channel(channel_id, port_id).await?;
                if !channel.is_none() {
                    return Err(anyhow::anyhow!("channel already exists"));
                }
                Ok(())
            }
        }
        impl<T: StateExt> Inner for T {}
    }
    impl<T: StateExt> ChannelOpenInitCheck for T {}
}

pub mod channel_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenTryCheck: StateExt + inner::Inner {
        async fn validate(&self, msg: &MsgChannelOpenTry) -> anyhow::Result<()> {
            let channel_id = ChannelId::new(self.get_channel_counter().await?);

            let connection = self.verify_connections_open(&msg).await?;

            // TODO: do we want to do capability authentication?

            let expected_channel = ChannelEnd {
                state: ChannelState::Init,
                ordering: msg.channel.ordering,
                remote: msg.channel.remote.clone(),
                connection_hops: vec![connection
                    .counterparty()
                    .connection_id
                    .clone()
                    .ok_or(anyhow::anyhow!("no counterparty connection id provided"))?],
                version: msg.counterparty_version.clone(),
            };

            // get the stored client state for the counterparty
            let trusted_client_state = self.get_client_state(connection.client_id()).await?;

            // check if the client is frozen
            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            // get the stored consensus state for the counterparty
            let trusted_consensus_state = self
                .get_verified_consensus_state(msg.proofs.height(), connection.client_id().clone())
                .await?;

            let client_def = AnyClient::from_client_type(trusted_client_state.client_type());

            // PROOF VERIFICATION. verify that our counterparty committed expected_channel to its
            // state.
            client_def.verify_channel_state(
                &trusted_client_state,
                msg.proofs.height(),
                &COMMITMENT_PREFIX.as_bytes().to_vec().try_into().unwrap(),
                msg.proofs.object_proof(),
                trusted_consensus_state.root(),
                &msg.port_id,
                &channel_id,
                &expected_channel,
            )?;

            Ok(())
        }
    }
    mod inner {
        use super::*;

        #[async_trait]
        pub trait Inner: StateExt {
            async fn verify_connections_open(
                &self,
                msg: &MsgChannelOpenTry,
            ) -> anyhow::Result<ConnectionEnd> {
                self.get_connection(&msg.channel.connection_hops[0])
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("connection not found"))
            }
        }
        impl<T: StateExt> Inner for T {}
    }
    impl<T: StateExt> ChannelOpenTryCheck for T {}
}
