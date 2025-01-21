use crate::component::proof_verification::{commit_acknowledgement, commit_packet};
use crate::prefix::MerklePrefixExt;
use crate::IBC_COMMITMENT_PREFIX;

use anyhow::Result;
use async_trait::async_trait;

use ibc_types::path::{
    AckPath, ChannelEndPath, CommitmentPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};

use cnidarium::{StateRead, StateWrite};
use ibc_types::core::channel::{ChannelEnd, ChannelId, Packet, PortId};
use penumbra_sdk_proto::{StateReadProto, StateWriteProto};

// Note: many of the methods on this trait need to write raw bytes,
// because the data they write is interpreted by counterparty chains.
#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_channel_counter(&mut self, counter: u64) {
        self.put_proto::<u64>("ibc_channel_counter".into(), counter);
    }

    async fn next_channel_id(&mut self) -> Result<ChannelId> {
        let ctr = self.get_channel_counter().await?;
        self.put_channel_counter(ctr + 1);

        Ok(ChannelId::new(ctr))
    }

    fn put_channel(&mut self, channel_id: &ChannelId, port_id: &PortId, channel: ChannelEnd) {
        self.put(
            IBC_COMMITMENT_PREFIX
                .apply_string(ChannelEndPath::new(port_id, channel_id).to_string()),
            channel,
        );
    }

    fn put_ack_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_raw(
            IBC_COMMITMENT_PREFIX.apply_string(SeqAckPath::new(port_id, channel_id).to_string()),
            sequence.to_be_bytes().to_vec(),
        );
    }

    fn put_recv_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_raw(
            IBC_COMMITMENT_PREFIX.apply_string(SeqRecvPath::new(port_id, channel_id).to_string()),
            sequence.to_be_bytes().to_vec(),
        );
    }

    fn put_send_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_raw(
            IBC_COMMITMENT_PREFIX.apply_string(SeqSendPath::new(port_id, channel_id).to_string()),
            sequence.to_be_bytes().to_vec(),
        );
    }

    fn put_packet_receipt(&mut self, packet: &Packet) {
        self.put_raw(
            IBC_COMMITMENT_PREFIX.apply_string(
                ReceiptPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence).to_string(),
            ),
            "1".into(),
        );
    }

    fn put_packet_commitment(&mut self, packet: &Packet) {
        let commitment_key = IBC_COMMITMENT_PREFIX.apply_string(
            CommitmentPath::new(&packet.port_on_a, &packet.chan_on_a, packet.sequence).to_string(),
        );
        let packet_hash = commit_packet(packet);

        self.put_raw(commitment_key, packet_hash);
    }

    fn delete_packet_commitment(
        &mut self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) {
        self.delete(
            IBC_COMMITMENT_PREFIX.apply_string(
                CommitmentPath::new(port_id, channel_id, sequence.into()).to_string(),
            ),
        );
    }

    fn put_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: u64,
        acknowledgement: &[u8],
    ) {
        self.put_raw(
            IBC_COMMITMENT_PREFIX
                .apply_string(AckPath::new(port_id, channel_id, sequence.into()).to_string()),
            commit_acknowledgement(acknowledgement),
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_channel_counter(&self) -> Result<u64> {
        self.get_proto::<u64>("ibc_channel_counter")
            .await
            .map(|counter| counter.unwrap_or(0))
    }

    async fn get_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
    ) -> Result<Option<ChannelEnd>> {
        self.get(
            &IBC_COMMITMENT_PREFIX
                .apply_string(ChannelEndPath::new(port_id, channel_id).to_string()),
        )
        .await
    }

    async fn get_recv_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        if let Some(be_bytes) = self
            .get_raw(
                &IBC_COMMITMENT_PREFIX
                    .apply_string(SeqRecvPath::new(port_id, channel_id).to_string()),
            )
            .await?
        {
            // Parse big endian bytes into u64
            Ok(u64::from_be_bytes(
                be_bytes
                    .try_into()
                    .expect("written values are always 8-byte arrays"),
            ))
        } else {
            // Default value for no key
            Ok(0)
        }
    }

    async fn get_ack_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        if let Some(be_bytes) = self
            .get_raw(
                &IBC_COMMITMENT_PREFIX
                    .apply_string(SeqAckPath::new(port_id, channel_id).to_string()),
            )
            .await?
        {
            // Parse big endian bytes into u64
            Ok(u64::from_be_bytes(
                be_bytes
                    .try_into()
                    .expect("written values are always 8-byte arrays"),
            ))
        } else {
            // Default value for no key
            Ok(0)
        }
    }

    async fn get_send_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        if let Some(be_bytes) = self
            .get_raw(
                &IBC_COMMITMENT_PREFIX
                    .apply_string(SeqSendPath::new(port_id, channel_id).to_string()),
            )
            .await?
        {
            // Parse big endian bytes into u64
            Ok(u64::from_be_bytes(
                be_bytes
                    .try_into()
                    .expect("written values are always 8-byte arrays"),
            ))
        } else {
            // Default value for no key
            Ok(0)
        }
    }

    async fn seen_packet(&self, packet: &Packet) -> Result<bool> {
        self.get_raw(&IBC_COMMITMENT_PREFIX.apply_string(
            ReceiptPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence).to_string(),
        ))
        .await
        .map(|res| res.is_some())
    }

    async fn seen_packet_by_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) -> Result<bool> {
        // TODO: make this logic more explicit
        self.get_raw(
            &IBC_COMMITMENT_PREFIX
                .apply_string(ReceiptPath::new(port_id, channel_id, sequence.into()).to_string()),
        )
        .await
        .map(|res| res.filter(|s| !s.is_empty()))
        .map(|res| res.is_some())
    }

    async fn get_packet_commitment(&self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_raw(
                &IBC_COMMITMENT_PREFIX.apply_string(
                    CommitmentPath::new(&packet.port_on_a, &packet.chan_on_a, packet.sequence)
                        .to_string(),
                ),
            )
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }

    async fn get_packet_commitment_by_id(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_raw(&IBC_COMMITMENT_PREFIX.apply_string(
                CommitmentPath::new(port_id, channel_id, sequence.into()).to_string(),
            ))
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }

    async fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: u64,
    ) -> Result<Option<Vec<u8>>> {
        self.get_raw(
            &IBC_COMMITMENT_PREFIX
                .apply_string(AckPath::new(port_id, channel_id, sequence.into()).to_string()),
        )
        .await
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
