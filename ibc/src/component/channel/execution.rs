pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelOpenInit) {
            let channel_id = self.next_channel_id().await.unwrap();
            let new_channel = ChannelEnd {
                state: ChannelState::Init,
                ordering: msg.channel.ordering,
                remote: msg.channel.remote.clone(),
                connection_hops: msg.channel.connection_hops.clone(),
                version: msg.channel.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id, new_channel)
                .await;
            self.put_send_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_recv_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_ack_sequence(&channel_id, &msg.port_id, 1).await;
        }
    }

    impl<T: StateExt> ChannelOpenInitExecute for T {}
}

pub mod channel_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenTryExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelOpenTry) {
            let channel_id = self.next_channel_id().await.unwrap();
            let new_channel = ChannelEnd {
                state: ChannelState::TryOpen,
                ordering: msg.channel.ordering,
                remote: msg.channel.remote.clone(),
                connection_hops: msg.channel.connection_hops.clone(),
                version: msg.channel.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id, new_channel)
                .await;
            self.put_send_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_recv_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_ack_sequence(&channel_id, &msg.port_id, 1).await;
        }
    }
    impl<T: StateExt> ChannelOpenTryExecute for T {}
}

pub mod channel_open_ack {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenAckExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelOpenAck) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Open);
            channel.set_version(msg.counterparty_version.clone());
            channel.set_counterparty_channel_id(msg.counterparty_channel_id);
            self.put_channel(&msg.channel_id, &msg.port_id, channel)
                .await;
        }
    }

    impl<T: StateExt> ChannelOpenAckExecute for T {}
}

pub mod channel_open_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenConfirmExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelOpenConfirm) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Open);
            self.put_channel(&msg.channel_id, &msg.port_id, channel)
                .await;
        }
    }

    impl<T: StateExt> ChannelOpenConfirmExecute for T {}
}

pub mod channel_close_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseInitExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelCloseInit) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();
            channel.set_state(ChannelState::Closed);
            self.put_channel(&msg.channel_id, &msg.port_id, channel)
                .await;
        }
    }

    impl<T: StateExt> ChannelCloseInitExecute for T {}
}

pub mod channel_close_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseConfirmExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelCloseConfirm) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Closed);
            self.put_channel(&msg.channel_id, &msg.port_id, channel)
                .await;
        }
    }

    impl<T: StateExt> ChannelCloseConfirmExecute for T {}
}

pub mod recv_packet {
    use super::super::*;

    #[async_trait]
    pub trait RecvPacketExecute: StateExt {
        async fn execute(&mut self, msg: &MsgRecvPacket) {
            let channel = self
                .get_channel(
                    &msg.packet.destination_channel,
                    &msg.packet.destination_port,
                )
                .await
                .unwrap()
                .unwrap();

            if channel.ordering == ChannelOrder::Ordered {
                let mut next_sequence_recv = self
                    .get_recv_sequence(
                        &msg.packet.destination_channel,
                        &msg.packet.destination_port,
                    )
                    .await
                    .unwrap();

                next_sequence_recv += 1;
                self.put_recv_sequence(
                    &msg.packet.destination_channel,
                    &msg.packet.destination_port,
                    next_sequence_recv,
                )
                .await;
            } else {
                // for unordered channels we must set the receipt so it can be verified on the other side
                // this receipt does not contain any data, since the packet has not yet been processed
                // it's just a single store key set to an empty string to indicate that the packet has been received
                self.put_packet_receipt(&msg.packet).await;
            }
        }
    }

    impl<T: StateExt> RecvPacketExecute for T {}
}
