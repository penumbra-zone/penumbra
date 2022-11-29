pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgChannelOpenInit) {
            let channel_id = self.next_channel_id().await.unwrap();
            let new_channel = ChannelEnd {
                state: ChannelState::Init,
                ordering: msg.chan_end_on_a.ordering,
                remote: msg.chan_end_on_a.remote.clone(),
                connection_hops: msg.chan_end_on_a.connection_hops.clone(),
                version: msg.chan_end_on_a.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id_on_a, new_channel.clone());
            self.put_send_sequence(&channel_id, &msg.port_id_on_a, 1);
            self.put_recv_sequence(&channel_id, &msg.port_id_on_a, 1);
            self.put_ack_sequence(&channel_id, &msg.port_id_on_a, 1);

            self.record(event::channel_open_init(
                &msg.port_id_on_a,
                &channel_id,
                &new_channel,
            ));
        }
    }

    impl<T: StateWriteExt> ChannelOpenInitExecute for T {}
}

pub mod channel_open_try {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenTryExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgChannelOpenTry) {
            let channel_id = self.next_channel_id().await.unwrap();
            let new_channel = ChannelEnd {
                state: ChannelState::TryOpen,
                ordering: msg.chan_end_on_b.ordering,
                remote: msg.chan_end_on_b.remote.clone(),
                connection_hops: msg.chan_end_on_b.connection_hops.clone(),
                version: msg.chan_end_on_b.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id_on_b, new_channel.clone());
            self.put_send_sequence(&channel_id, &msg.port_id_on_b, 1);
            self.put_recv_sequence(&channel_id, &msg.port_id_on_b, 1);
            self.put_ack_sequence(&channel_id, &msg.port_id_on_b, 1);

            self.record(event::channel_open_try(
                &msg.port_id_on_b,
                &channel_id,
                &new_channel,
            ));
        }
    }

    impl<T: StateWriteExt> ChannelOpenTryExecute for T {}
}

pub mod channel_open_ack {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenAckExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgChannelOpenAck) {
            let mut channel = self
                .get_channel(&msg.chan_id_on_a, &msg.port_id_on_a)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Open);
            channel.set_version(msg.version_on_b.clone());
            channel.set_counterparty_channel_id(msg.chan_id_on_b);
            self.put_channel(&msg.chan_id_on_a, &msg.port_id_on_a, channel.clone());

            self.record(event::channel_open_ack(
                &msg.port_id_on_a,
                &msg.chan_id_on_a,
                &channel,
            ));
        }
    }

    impl<T: StateWriteExt> ChannelOpenAckExecute for T {}
}

pub mod channel_open_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenConfirmExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgChannelOpenConfirm) {
            let mut channel = self
                .get_channel(&msg.chan_id_on_b, &msg.port_id_on_b)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Open);
            self.put_channel(&msg.chan_id_on_b, &msg.port_id_on_b, channel.clone());

            self.record(event::channel_open_confirm(
                &msg.port_id_on_b,
                &msg.chan_id_on_b,
                &channel,
            ));
        }
    }

    impl<T: StateWriteExt> ChannelOpenConfirmExecute for T {}
}

pub mod channel_close_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseInitExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgChannelCloseInit) {
            let mut channel = self
                .get_channel(&msg.chan_id_on_a, &msg.port_id_on_a)
                .await
                .unwrap()
                .unwrap();
            channel.set_state(ChannelState::Closed);
            self.put_channel(&msg.chan_id_on_a, &msg.port_id_on_a, channel.clone());

            self.record(event::channel_close_init(
                &msg.port_id_on_a,
                &msg.chan_id_on_a,
                &channel,
            ));
        }
    }

    impl<T: StateWriteExt> ChannelCloseInitExecute for T {}
}

pub mod channel_close_confirm {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseConfirmExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgChannelCloseConfirm) {
            let mut channel = self
                .get_channel(&msg.chan_id_on_b, &msg.port_id_on_b)
                .await
                .unwrap()
                .unwrap();

            channel.set_state(ChannelState::Closed);
            self.put_channel(&msg.chan_id_on_b, &msg.port_id_on_b, channel.clone());

            self.record(event::channel_close_confirm(
                &msg.port_id_on_b,
                &msg.chan_id_on_b,
                &channel,
            ));
        }
    }

    impl<T: StateWriteExt> ChannelCloseConfirmExecute for T {}
}

pub mod recv_packet {
    use super::super::*;

    #[async_trait]
    pub trait RecvPacketExecute: StateWriteExt {
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
                );
            } else {
                // for unordered channels we must set the receipt so it can be verified on the other side
                // this receipt does not contain any data, since the packet has not yet been processed
                // it's just a single store key set to an empty string to indicate that the packet has been received
                self.put_packet_receipt(&msg.packet);
            }

            self.record(event::receive_packet(&msg.packet, &channel));
        }
    }

    impl<T: StateWriteExt> RecvPacketExecute for T {}
}

pub mod acknowledge_packet {
    use super::super::*;

    #[async_trait]
    pub trait AcknowledgePacketExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgAcknowledgement) {
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
                );
            }

            // delete our commitment so we can't ack it again
            self.delete_packet_commitment(
                &msg.packet.source_channel,
                &msg.packet.source_port,
                msg.packet.sequence.into(),
            );

            self.record(event::acknowledge_packet(&msg.packet, &channel));
        }
    }

    impl<T: StateWriteExt> AcknowledgePacketExecute for T {}
}

pub mod timeout {
    use super::super::*;

    #[async_trait]
    pub trait TimeoutExecute: StateWriteExt {
        async fn execute(&mut self, msg: &MsgTimeout) {
            let mut channel = self
                .get_channel(&msg.packet.source_channel, &msg.packet.source_port)
                .await
                .unwrap()
                .unwrap();

            self.delete_packet_commitment(
                &msg.packet.source_channel,
                &msg.packet.source_port,
                msg.packet.sequence.into(),
            );

            if channel.ordering == ChannelOrder::Ordered {
                // if the channel is ordered and we get a timeout packet, close the channel
                channel.set_state(ChannelState::Closed);
                self.put_channel(
                    &msg.packet.source_channel,
                    &msg.packet.source_port,
                    channel.clone(),
                );
            }

            self.record(event::timeout_packet(&msg.packet, &channel));
        }
    }

    impl<T: StateWriteExt> TimeoutExecute for T {}
}
