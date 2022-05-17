mod proof_verification {
    use super::super::*;
    use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
    use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
    use ibc::core::ics23_commitment::commitment::CommitmentRoot;
    use ibc::core::ics23_commitment::merkle::apply_prefix;
    use ibc::core::ics23_commitment::merkle::MerkleProof;
    use ibc::core::ics23_commitment::specs::ProofSpecs;
    use ibc::core::ics24_host::Path;
    use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;

    use crate::component::client::View as _;
    use ibc::core::ics02_client::client_consensus::AnyConsensusState;
    use ibc::core::ics02_client::client_state::AnyClientState;
    use ibc::core::ics04_channel::context::calculate_block_delay;
    use ibc::core::ics24_host::identifier::ClientId;
    use ibc::core::ics24_host::path::CommitmentsPath;
    use ibc::downcast;
    use penumbra_chain::View as _;
    use sha2::{Digest, Sha256};

    // NOTE: this is underspecified.
    // using the same implementation here as ibc-go:
    // https://github.com/cosmos/ibc-go/blob/main/modules/core/04-channel/types/packet.go#L19
    // timeout_timestamp + timeout_height.revision_number + timeout_height.revision_height
    // + sha256(data)
    fn commit_packet(packet: &Packet) -> Vec<u8> {
        let mut commit = vec![];
        commit.extend_from_slice(&packet.timeout_timestamp.nanoseconds().to_be_bytes());
        commit.extend_from_slice(&packet.timeout_height.revision_number.to_be_bytes());
        commit.extend_from_slice(&packet.timeout_height.revision_height.to_be_bytes());
        commit.extend_from_slice(&Sha256::digest(&packet.data));

        Sha256::digest(&commit).to_vec()
    }

    fn verify_merkle_proof(
        proof_specs: &ProofSpecs,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: impl Into<Path>,
        value: Vec<u8>,
    ) -> anyhow::Result<()> {
        let merkle_path = apply_prefix(prefix, vec![path.into().to_string()]);
        let merkle_proof: MerkleProof = RawMerkleProof::try_from(proof.clone())
            .map_err(|_| anyhow::anyhow!("invalid merkle proof"))?
            .into();

        merkle_proof.verify_membership(&proof_specs, root.clone().into(), merkle_path, value, 0)?;

        Ok(())
    }

    #[async_trait]
    pub trait ChannelProofVerifier: StateExt {
        async fn verify_channel_proof(
            &self,
            connection: &ConnectionEnd,
            proofs: &ibc::proofs::Proofs,
            channel_id: &ChannelId,
            port_id: &PortId,
            expected_channel: &ChannelEnd,
        ) -> anyhow::Result<()> {
            // get the stored client state for the counterparty
            let trusted_client_state = self.get_client_state(connection.client_id()).await?;

            // check if the client is frozen
            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            // get the stored consensus state for the counterparty
            let trusted_consensus_state = self
                .get_verified_consensus_state(proofs.height(), connection.client_id().clone())
                .await?;

            let client_def = AnyClient::from_client_type(trusted_client_state.client_type());

            // PROOF VERIFICATION. verify that our counterparty committed expected_channel to its
            // state.
            client_def.verify_channel_state(
                &trusted_client_state,
                proofs.height(),
                connection.counterparty().prefix(),
                proofs.object_proof(),
                trusted_consensus_state.root(),
                port_id,
                channel_id,
                expected_channel,
            )?;

            Ok(())
        }
    }

    #[async_trait]
    pub trait PacketProofVerifier: StateExt {
        async fn verify_packet_data(
            &self,
            client_id: &ClientId,
            client_state: &AnyClientState,
            connection: &ConnectionEnd,
            packet: &Packet,
            proofs: &ibc::proofs::Proofs,
            trusted_consensus_state: &AnyConsensusState,
        ) -> anyhow::Result<()> {
            // currently only tendermint clients.
            let tm_client_state = downcast!(client_state => AnyClientState::Tendermint)
                .ok_or(anyhow::anyhow!("client state is not tendermint"))?;

            tm_client_state.verify_height(proofs.height())?;
            let current_timestamp = self.get_block_timestamp().await?;
            let current_height = self.get_block_height().await?;

            let processed_height = self
                .get_client_update_height(client_id, &proofs.height())
                .await?;

            let processed_time = self
                .get_client_update_time(client_id, &proofs.height())
                .await?;

            // NOTE: hardcoded for now, should be a chain parameter.
            let max_time_per_block = std::time::Duration::from_secs(20);

            let delay_period_time = connection.delay_period();
            let delay_period_blocks = calculate_block_delay(delay_period_time, max_time_per_block);

            /*
            if current_timestamp < processed_time + delay_period_time {
                return Err(anyhow::anyhow!("not enough time has passed for packet"));
            }
            if current_height < processed_height + delay_period_blocks {
                return Err(anyhow::anyhow!("not enough blocks have passed for packet"));
            }*/

            let commitment_path = CommitmentsPath {
                port_id: packet.destination_port.clone(),
                channel_id: packet.destination_channel.clone(),
                sequence: packet.sequence,
            };

            let commitment_bytes = commit_packet(&packet);

            verify_merkle_proof(
                &tm_client_state.proof_specs,
                connection.counterparty().prefix(),
                proofs.object_proof(),
                trusted_consensus_state.root(),
                commitment_path,
                commitment_bytes,
            )?;

            Ok(())
        }

        async fn verify_packet_proof(
            &self,
            connection: &ConnectionEnd,
            proofs: &ibc::proofs::Proofs,
            packet: &Packet,
        ) -> anyhow::Result<()> {
            // get the stored client state for the counterparty
            let trusted_client_state = self.get_client_state(connection.client_id()).await?;

            // check if the client is frozen
            // TODO: should we also check if the client is expired here?
            if trusted_client_state.is_frozen() {
                return Err(anyhow::anyhow!("client is frozen"));
            }

            // get the stored consensus state for the counterparty
            let trusted_consensus_state = self
                .get_verified_consensus_state(proofs.height(), connection.client_id().clone())
                .await?;

            self.verify_packet_data(
                &connection.client_id(),
                &trusted_client_state,
                connection,
                packet,
                proofs,
                &trusted_consensus_state,
            )
            .await?;

            Ok(())
        }
    }

    impl<T: StateExt> ChannelProofVerifier for T {}
    impl<T: StateExt> PacketProofVerifier for T {}
}

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
            self.verify_connections_exist(msg).await?;

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
                if channel.is_some() {
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
    use super::proof_verification::ChannelProofVerifier;

    #[async_trait]
    pub trait ChannelOpenTryCheck: StateExt + inner::Inner {
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
        pub trait Inner: StateExt {
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
        impl<T: StateExt> Inner for T {}
    }
    impl<T: StateExt> ChannelOpenTryCheck for T {}
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
    pub trait ChannelOpenAckCheck: StateExt + inner::Inner {
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
        pub trait Inner: StateExt {
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
        impl<T: StateExt> Inner for T {}
    }

    impl<T: StateExt> ChannelOpenAckCheck for T {}
}

pub mod channel_open_confirm {
    use super::super::*;
    use super::proof_verification::ChannelProofVerifier;

    #[async_trait]
    pub trait ChannelOpenConfirmCheck: StateExt {
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

    impl<T: StateExt> ChannelOpenConfirmCheck for T {}
}

pub mod channel_close_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelCloseInitCheck: StateExt {
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

    impl<T: StateExt> ChannelCloseInitCheck for T {}
}

pub mod channel_close_confirm {
    use super::super::*;
    use super::proof_verification::ChannelProofVerifier;

    #[async_trait]
    pub trait ChannelCloseConfirmCheck: StateExt {
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

    impl<T: StateExt> ChannelCloseConfirmCheck for T {}
}

mod packet_validation {
    use super::super::*;

    #[async_trait]
    pub trait PacketValidation {}

    impl<T: StateExt> PacketValidation for T {}
}

pub mod recv_packet {
    use super::super::*;
    use ibc::timestamp::Timestamp as IBCTimestamp;
    use ibc::Height as IBCHeight;
    use penumbra_chain::View as _;

    #[async_trait]
    pub trait RecvPacketCheck: StateExt {
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
                    .ok_or(anyhow::anyhow!("missing channel id"))?
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
                    < msg.packet.timeout_height
            {
                return Err(anyhow::anyhow!("packet has timed out"));
            }

            if msg.packet.timeout_timestamp != IBCTimestamp::none()
                && self.get_block_timestamp().await?
                    < msg
                        .packet
                        .timeout_timestamp
                        .into_tm_time()
                        .ok_or(anyhow::anyhow!("invalid timestamp"))?
            {
                return Err(anyhow::anyhow!("packet has timed out"));
            }

            Ok(())
        }
    }

    impl<T: StateExt> RecvPacketCheck for T {}
}
