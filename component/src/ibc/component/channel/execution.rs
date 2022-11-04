pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgChannelOpenInit) {
            let channel_id = self.next_channel_id().await.unwrap();
            let new_channel = ChannelEnd {
                state: ChannelState::Init,
                ordering: msg.channel.ordering,
                remote: msg.channel.remote.clone(),
                connection_hops: msg.channel.connection_hops.clone(),
                version: msg.channel.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id, new_channel.clone())
                .await;
            self.put_send_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_recv_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_ack_sequence(&channel_id, &msg.port_id, 1).await;

            state.record(event::channel_open_init(
                &msg.port_id,
                &channel_id,
                &new_channel,
            ));
        }
    }
}

pub mod channel_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenTryExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgChannelOpenTry) {
            let channel_id = self.next_channel_id().await.unwrap();
            let new_channel = ChannelEnd {
                state: ChannelState::TryOpen,
                ordering: msg.channel.ordering,
                remote: msg.channel.remote.clone(),
                connection_hops: msg.channel.connection_hops.clone(),
                version: msg.channel.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id, new_channel.clone())
                .await;
            self.put_send_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_recv_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_ack_sequence(&channel_id, &msg.port_id, 1).await;

            state.record(event::channel_open_try(
                &msg.port_id,
                &channel_id,
                &new_channel,
            ));
        }
    }
}

pub mod channel_open_ack {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenAckExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgChannelOpenAck) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Open);
            channel.set_version(msg.counterparty_version.clone());
            channel.set_counterparty_channel_id(msg.counterparty_channel_id);
            self.put_channel(&msg.channel_id, &msg.port_id, channel.clone())
                .await;

            state.record(event::channel_open_ack(
                &msg.port_id,
                &msg.channel_id,
                &channel,
            ));
        }
    }
}

pub mod channel_open_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenConfirmExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgChannelOpenConfirm) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Open);
            self.put_channel(&msg.channel_id, &msg.port_id, channel.clone())
                .await;

            state.record(event::channel_open_confirm(
                &msg.port_id,
                &msg.channel_id,
                &channel,
            ));
        }
    }
}

pub mod channel_close_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseInitExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgChannelCloseInit) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();
            channel.set_state(ChannelState::Closed);
            self.put_channel(&msg.channel_id, &msg.port_id, channel.clone())
                .await;

            state.record(event::channel_close_init(
                &msg.port_id,
                &msg.channel_id,
                &channel,
            ));
        }
    }
}

pub mod channel_close_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseConfirmExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgChannelCloseConfirm) {
            let mut channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Closed);
            self.put_channel(&msg.channel_id, &msg.port_id, channel.clone())
                .await;

            state.record(event::channel_close_confirm(
                &msg.port_id,
                &msg.channel_id,
                &channel,
            ));
        }
    }
}

pub mod recv_packet {
    use super::super::*;

    #[async_trait]
    pub trait RecvPacketExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgRecvPacket) {
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

            state.record(event::receive_packet(&msg.packet, &channel));
        }
    }
}

pub mod acknowledge_packet {
    use super::super::*;

    #[async_trait]
    pub trait AcknowledgePacketExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgAcknowledgement) {
            let channel = self
                .get_channel(&msg.packet.source_channel, &msg.packet.source_port)
                .await
                .unwrap()
                .unwrap();

            if channel.ordering == ChannelOrder::Ordered {
                let mut next_sequence_ack = self
                    .get_ack_sequence(&msg.packet.source_channel, &msg.packet.source_port)
                    .await
                    .unwrap();
                next_sequence_ack += 1;
                self.put_ack_sequence(
                    &msg.packet.source_channel,
                    &msg.packet.source_port,
                    next_sequence_ack,
                )
                .await;
            }

            // delete our commitment so we can't ack it again
            self.delete_packet_commitment(
                &msg.packet.source_channel,
                &msg.packet.source_port,
                msg.packet.sequence.into(),
            )
            .await;

            state.record(event::acknowledge_packet(&msg.packet, &channel));
        }
    }
}

pub mod timeout {
    use super::super::*;

    #[async_trait]
    pub trait TimeoutExecute {
        async fn execute(&mut self, ctx: Context, msg: &MsgTimeout) {
            let mut channel = self
                .get_channel(&msg.packet.source_channel, &msg.packet.source_port)
                .await
                .unwrap()
                .unwrap();

            self.delete_packet_commitment(
                &msg.packet.source_channel,
                &msg.packet.source_port,
                msg.packet.sequence.into(),
            )
            .await;

            if channel.ordering == ChannelOrder::Ordered {
                // if the channel is ordered and we get a timeout packet, close the channel
                channel.set_state(ChannelState::Closed);
                self.put_channel(
                    &msg.packet.source_channel,
                    &msg.packet.source_port,
                    channel.clone(),
                )
                .await;
            }

            state.record(event::timeout_packet(&msg.packet, &channel));
        }
    }
}
