pub mod proof_verification;

pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitCheck: StateReadExt + inner::Inner {
        async fn validate(&self, msg: &MsgChannelOpenInit) -> anyhow::Result<()> {
            let channel_id = self.get_channel_id().await?;

            self.verify_channel_does_not_exist(&channel_id, &msg.port_id)
                .await?;

            // NOTE: optimistic channel handshakes are allowed, so we don't check if the connection
            // is open here.
            self.verify_connections_exist(msg).await?;

            // TODO: do we want to do capability authentication?

            Ok(())
        }
    }
    mod inner {
        use super::*;

        #[async_trait]
        pub trait Inner: StateReadExt {
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
                if channel.is_some() {
                    return Err(anyhow::anyhow!("channel already exists"));
                }
                Ok(())
            }
        }
        impl<T: StateReadExt> Inner for T {}
    }
    impl<T: StateReadExt> ChannelOpenInitCheck for T {}
}

pub mod channel_open_try {
    use super::super::*;
    use super::proof_verification::ChannelProofVerifier;

    #[async_trait]
    pub trait ChannelOpenTryCheck: StateReadExt + inner::Inner {
        async fn validate(&self, msg: &MsgChannelOpenTry) -> anyhow::Result<()> {
            let channel_id = ChannelId::new(self.get_channel_counter().await?);

            let connection = self.verify_connections_open(msg).await?;

            // TODO: do we want to do capability authentication?
            // TODO: version intersection

            let expected_counterparty = Counterparty::new(msg.port_id.clone(), None);

            let expected_channel = ChannelEnd {
                state: ChannelState::Init,
                ordering: msg.channel.ordering,
                remote: expected_counterparty,
                connection_hops: vec![connection
                    .counterparty()
                    .connection_id
                    .clone()
                    .ok_or_else(|| anyhow::anyhow!("no counterparty connection id provided"))?],
                version: msg.counterparty_version.clone(),
            };

            self.verify_channel_proof(
                &connection,
                &msg.proofs,
                &channel_id,
                &msg.port_id,
                &expected_channel,
            )
            .await
        }
    }
    mod inner {
        use super::*;

        #[async_trait]
        pub trait Inner: StateReadExt {
            async fn verify_connections_open(
                &self,
                msg: &MsgChannelOpenTry,
            ) -> anyhow::Result<ConnectionEnd> {
                let connection = self
                    .get_connection(&msg.channel.connection_hops[0])
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("connection not found"))?;

                if connection.state != ConnectionState::Open {
                    Err(anyhow::anyhow!("connection for channel is not open"))
                } else {
                    Ok(connection)
                }
            }
        }
        impl<T: StateReadExt> Inner for T {}
    }
    impl<T: StateReadExt> ChannelOpenTryCheck for T {}
}

pub mod channel_open_ack {
    use super::super::*;
    use super::proof_verification::ChannelProofVerifier;

    fn channel_state_is_correct(channel: &ChannelEnd) -> anyhow::Result<()> {
        if channel.state == ChannelState::Init || channel.state == ChannelState::TryOpen {
            Ok(())
        } else {
            Err(anyhow::anyhow!("channel is not in the correct state"))
        }
    }

    #[async_trait]
    pub trait ChannelOpenAckCheck: inner::Inner {
        async fn validate(&self, msg: &MsgChannelOpenAck) -> anyhow::Result<()> {
            let channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;

            channel_state_is_correct(&channel)?;

            // TODO: capability authentication?

            let connection = self.verify_channel_connection_open(&channel).await?;

            let expected_counterparty =
                Counterparty::new(msg.port_id.clone(), Some(msg.channel_id));

            let expected_connection_hops = vec![connection
                .counterparty()
                .connection_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("no counterparty connection id provided"))?];

            let expected_channel = ChannelEnd {
                state: ChannelState::TryOpen,
                ordering: channel.ordering,
                remote: expected_counterparty,
                connection_hops: expected_connection_hops,
                version: msg.counterparty_version.clone(),
            };

            self.verify_channel_proof(
                &connection,
                &msg.proofs,
                &msg.counterparty_channel_id,
                &channel.remote.port_id,
                &expected_channel,
            )
            .await
        }
    }
    mod inner {
        use super::*;

        #[async_trait]
        pub trait Inner {
            async fn verify_channel_connection_open(
                &self,
                channel: &ChannelEnd,
            ) -> anyhow::Result<ConnectionEnd> {
                let connection = self
                    .get_connection(&channel.connection_hops[0])
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;

                if connection.state != ConnectionState::Open {
                    Err(anyhow::anyhow!("connection for channel is not open"))
                } else {
                    Ok(connection)
                }
            }
        }
        impl<T: StateReadExt> Inner for T {}
    }
    impl<T: StateReadExt> ChannelOpenAckCheck for T {}
}

pub mod channel_open_confirm {
    use super::super::*;
    use super::proof_verification::ChannelProofVerifier;

    #[async_trait]
    pub trait ChannelOpenConfirmCheck {
        async fn validate(&self, msg: &MsgChannelOpenConfirm) -> anyhow::Result<()> {
            let channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
            if !channel.state_matches(&ChannelState::TryOpen) {
                return Err(anyhow::anyhow!("channel is not in the correct state"));
            }

            // TODO: capability authentication?

            let connection = self
                .get_connection(&channel.connection_hops[0])
                .await?
                .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
            if !connection.state_matches(&ConnectionState::Open) {
                return Err(anyhow::anyhow!("connection for channel is not open"));
            }

            let expected_connection_hops = vec![connection
                .counterparty()
                .connection_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("no counterparty connection id provided"))?];

            let expected_counterparty =
                Counterparty::new(msg.port_id.clone(), Some(msg.channel_id));

            let expected_channel = ChannelEnd {
                state: ChannelState::Open,
                ordering: channel.ordering,
                remote: expected_counterparty,
                connection_hops: expected_connection_hops,
                version: channel.version.clone(),
            };

            self.verify_channel_proof(
                &connection,
                &msg.proofs,
                &channel
                    .remote
                    .channel_id
                    .ok_or_else(|| anyhow::anyhow!("no channel id"))?,
                &channel.remote.port_id,
                &expected_channel,
            )
            .await
        }
    }

    impl<T: StateReadExt> ChannelOpenConfirmCheck for T {}
}

pub mod channel_close_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseInitCheck: StateReadExt {
        async fn validate(&self, msg: &MsgChannelCloseInit) -> anyhow::Result<()> {
            // TODO: capability authentication?
            //
            // we probably do need capability authentication here, or some other authorization
            // method, to prevent anyone from spuriously closing channels.
            //
            let channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
            if channel.state_matches(&ChannelState::Closed) {
                return Err(anyhow::anyhow!("channel is already closed"));
            }

            let connection = self
                .get_connection(&channel.connection_hops[0])
                .await?
                .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
            if !connection.state_matches(&ConnectionState::Open) {
                return Err(anyhow::anyhow!("connection for channel is not open"));
            }

            Ok(())
        }
    }

    impl<T: StateReadExt> ChannelCloseInitCheck for T {}
}

pub mod channel_close_confirm {
    use super::super::*;
    use super::proof_verification::ChannelProofVerifier;

    #[async_trait]
    pub trait ChannelCloseConfirmCheck {
        async fn validate(&self, msg: &MsgChannelCloseConfirm) -> anyhow::Result<()> {
            // TODO: capability authentication?
            //
            // we probably do need capability authentication here, or some other authorization
            // method, to prevent anyone from spuriously closing channels.
            //
            let channel = self
                .get_channel(&msg.channel_id, &msg.port_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
            if channel.state_matches(&ChannelState::Closed) {
                return Err(anyhow::anyhow!("channel is already closed"));
            }

            let connection = self
                .get_connection(&channel.connection_hops[0])
                .await?
                .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
            if !connection.state_matches(&ConnectionState::Open) {
                return Err(anyhow::anyhow!("connection for channel is not open"));
            }

            let expected_connection_hops = vec![connection
                .counterparty()
                .connection_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("no counterparty connection id provided"))?];

            let expected_counterparty =
                Counterparty::new(msg.port_id.clone(), Some(msg.channel_id));

            let expected_channel = ChannelEnd {
                state: ChannelState::Closed,
                ordering: channel.ordering,
                remote: expected_counterparty,
                connection_hops: expected_connection_hops,
                version: channel.version.clone(),
            };

            self.verify_channel_proof(
                &connection,
                &msg.proofs,
                &channel
                    .remote
                    .channel_id
                    .ok_or_else(|| anyhow::anyhow!("no channel id"))?,
                &channel.remote.port_id,
                &expected_channel,
            )
            .await
        }
    }

    impl<T: StateReadExt> ChannelCloseConfirmCheck for T {}
}

pub mod recv_packet {
    use super::super::*;
    use super::proof_verification::PacketProofVerifier;
    use ibc::timestamp::Timestamp as IBCTimestamp;
    use ibc::Height as IBCHeight;
    use penumbra_chain::StateReadExt as _;

    #[async_trait]
    pub trait RecvPacketCheck: StateReadExt {
        async fn validate(&self, msg: &MsgRecvPacket) -> anyhow::Result<()> {
            let channel = self
                .get_channel(
                    &msg.packet.destination_channel,
                    &msg.packet.destination_port,
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
            if !channel.state_matches(&ChannelState::Open) {
                return Err(anyhow::anyhow!("channel is not open"));
            }

            // TODO: capability authentication?

            if msg.packet.source_port != channel.counterparty().port_id {
                return Err(anyhow::anyhow!("packet source port does not match channel"));
            }
            if msg.packet.source_channel
                != channel
                    .counterparty()
                    .channel_id
                    .ok_or_else(|| anyhow::anyhow!("missing channel id"))?
            {
                return Err(anyhow::anyhow!(
                    "packet source channel does not match channel"
                ));
            }

            let connection = self
                .get_connection(&channel.connection_hops[0])
                .await?
                .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
            if !connection.state_matches(&ConnectionState::Open) {
                return Err(anyhow::anyhow!("connection for channel is not open"));
            }

            if msg.packet.timeout_height != IBCHeight::zero()
                && IBCHeight::zero().with_revision_height(self.get_block_height().await?)
                    >= msg.packet.timeout_height
            {
                return Err(anyhow::anyhow!("packet has timed out"));
            }

            if msg.packet.timeout_timestamp != IBCTimestamp::none()
                && self.get_block_timestamp().await?
                    >= msg
                        .packet
                        .timeout_timestamp
                        .into_tm_time()
                        .ok_or_else(|| anyhow::anyhow!("invalid timestamp"))?
            {
                return Err(anyhow::anyhow!("packet has timed out"));
            }

            self.verify_packet_recv_proof(&connection, msg).await?;

            if channel.ordering == ChannelOrder::Ordered {
                let next_sequence_recv = self
                    .get_recv_sequence(
                        &msg.packet.destination_channel,
                        &msg.packet.destination_port,
                    )
                    .await?;

                if msg.packet.sequence != next_sequence_recv.into() {
                    return Err(anyhow::anyhow!("packet sequence number does not match"));
                }
            } else if self.seen_packet(&msg.packet).await? {
                return Err(anyhow::anyhow!("packet has already been processed"));
            }

            Ok(())
        }
    }

    impl<T: StateReadExt> RecvPacketCheck for T {}
}

pub mod acknowledge_packet {
    use super::super::*;
    use super::proof_verification::commit_packet;
    use super::proof_verification::PacketProofVerifier;

    #[async_trait]
    pub trait AcknowledgePacketCheck: StateReadExt {
        async fn validate(&self, msg: &MsgAcknowledgement) -> anyhow::Result<()> {
            let channel = self
                .get_channel(&msg.packet.source_channel, &msg.packet.source_port)
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
            if !channel.state_matches(&ChannelState::Open) {
                return Err(anyhow::anyhow!("channel is not open"));
            }

            // TODO: capability authentication?

            if msg.packet.destination_port != channel.counterparty().port_id {
                return Err(anyhow::anyhow!(
                    "packet destination port does not match channel"
                ));
            }

            if msg.packet.destination_channel
                != channel
                    .counterparty()
                    .channel_id
                    .ok_or_else(|| anyhow::anyhow!("missing counterparty channel id"))?
            {
                return Err(anyhow::anyhow!(
                    "packet destination channel does not match channel"
                ));
            }

            let connection = self
                .get_connection(&channel.connection_hops[0])
                .await?
                .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;
            if !connection.state_matches(&ConnectionState::Open) {
                return Err(anyhow::anyhow!("connection for channel is not open"));
            }

            // verify we sent this packet and haven't cleared it yet
            let commitment = self
                .get_packet_commitment(&msg.packet)
                .await?
                .ok_or_else(|| anyhow::anyhow!("packet commitment not found"))?;
            if commitment != commit_packet(&msg.packet) {
                return Err(anyhow::anyhow!("packet commitment does not match"));
            }

            self.verify_packet_ack_proof(&connection, msg).await?;

            if channel.ordering == ChannelOrder::Ordered {
                let next_sequence_ack = self
                    .get_ack_sequence(&msg.packet.source_channel, &msg.packet.source_port)
                    .await?;
                if msg.packet.sequence != next_sequence_ack.into() {
                    return Err(anyhow::anyhow!("packet sequence number does not match"));
                }
            }

            Ok(())
        }
    }

    impl<T: StateReadExt> AcknowledgePacketCheck for T {}
}

pub mod timeout {
    use super::super::*;
    use super::proof_verification::commit_packet;
    use super::proof_verification::PacketProofVerifier;
    use ibc::timestamp::Timestamp as IBCTimestamp;

    #[async_trait]
    pub trait TimeoutCheck: StateReadExt {
        async fn validate(&self, msg: &MsgTimeout) -> anyhow::Result<()> {
            let channel = self
                .get_channel(&msg.packet.source_channel, &msg.packet.source_port)
                .await?
                .ok_or_else(|| anyhow::anyhow!("channel not found"))?;
            if !channel.state_matches(&ChannelState::Open) {
                return Err(anyhow::anyhow!("channel is not open"));
            }

            // TODO: capability authentication?
            if msg.packet.destination_channel
                != channel
                    .counterparty()
                    .channel_id
                    .ok_or_else(|| anyhow::anyhow!("missing channel id"))?
            {
                return Err(anyhow::anyhow!(
                    "packet destination channel does not match channel"
                ));
            }
            if msg.packet.destination_port != channel.counterparty().port_id {
                return Err(anyhow::anyhow!(
                    "packet destination port does not match channel"
                ));
            }

            let connection = self
                .get_connection(&channel.connection_hops[0])
                .await?
                .ok_or_else(|| anyhow::anyhow!("connection not found for channel"))?;

            // check that timeout height or timeout timestamp has passed on the other end
            if msg.packet.timeout_height == ibc::Height::zero()
                || msg.proofs.height() < msg.packet.timeout_height
            {
                return Err(anyhow::anyhow!(
                    "packet has not timed out on the counterparty chain"
                ));
            }
            if msg.packet.timeout_timestamp == IBCTimestamp::none()
                || self
                    .get_client_update_time(connection.client_id(), &msg.proofs.height())
                    .await?
                    .nanoseconds()
                    < msg.packet.timeout_timestamp.nanoseconds()
            {
                return Err(anyhow::anyhow!(
                    "packet has not timed out on the counterparty chain"
                ));
            }

            // verify that we actually sent this packet
            let commitment = self
                .get_packet_commitment(&msg.packet)
                .await?
                .ok_or_else(|| anyhow::anyhow!("packet commitment not found"))?;
            if commitment != commit_packet(&msg.packet) {
                return Err(anyhow::anyhow!("packet commitment does not match"));
            }

            if channel.ordering == ChannelOrder::Ordered {
                // ordered channel: check that packet has not been received
                if msg.next_sequence_recv > msg.packet.sequence {
                    return Err(anyhow::anyhow!("packet sequence number does not match"));
                }

                // in the case of a timed-out ordered packet, the counterparty should have
                // committed the next sequence number to their state
                self.verify_packet_timeout_proof(&connection, msg).await?;
            } else {
                // in the case of a timed-out unordered packet, the counterparty should not have
                // committed a receipt to the state.
                self.verify_packet_timeout_absence_proof(&connection, msg)
                    .await?;
            }

            Ok(())
        }
    }

    impl<T: StateReadExt> TimeoutCheck for T {}
}
