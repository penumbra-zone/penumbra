pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitCheck: StateExt + inner::Inner {
        async fn validate(&self, msg: &MsgChannelOpenInit) -> anyhow::Result<()> {
            let channel_id = self.get_channel_id().await?;

            self.verify_channel_does_not_exist(&channel_id, &msg.port_id)
                .await?;

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
