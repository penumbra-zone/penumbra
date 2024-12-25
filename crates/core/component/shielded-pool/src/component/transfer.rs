use std::str::FromStr;

use crate::{
    component::{AssetRegistry, NoteManager},
    event::{self, FungibleTokenTransferPacketMetadata},
    Ics20Withdrawal,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::core::channel::Packet;
use ibc_types::{
    core::channel::{
        channel::Order as ChannelOrder,
        msgs::{
            MsgAcknowledgement, MsgChannelCloseConfirm, MsgChannelCloseInit, MsgChannelOpenAck,
            MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgRecvPacket,
            MsgTimeout,
        },
        ChannelId, PortId, Version,
    },
    transfer::acknowledgement::TokenTransferAcknowledgement,
};
use penumbra_sdk_asset::{asset, asset::Metadata, Value};
use penumbra_sdk_ibc::component::ChannelStateReadExt;
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::{
    penumbra::core::component::ibc::v1::FungibleTokenPacketData, DomainType as _, StateReadProto,
    StateWriteProto,
};
use penumbra_sdk_sct::CommitmentSource;

use penumbra_sdk_ibc::component::{
    app_handler::{AppHandler, AppHandlerCheck, AppHandlerExecute},
    packet::{
        IBCPacket, SendPacketRead as _, SendPacketWrite as _, Unchecked, WriteAcknowledgement as _,
    },
    state_key,
};
use tendermint::Time;

// returns a bool indicating if the provided denom was issued locally or if it was bridged in.
// this logic is a bit tricky, and adapted from https://github.com/cosmos/ibc/tree/main/spec/app/ics-020-fungible-token-transfer (sendFungibleTokens).
//
// what we want to do is to determine if the denom being withdrawn is a native token (one
// that originates from Penumbra) or a bridged token (one that was sent into penumbra from
// IBC).
//
// A simple way of doing this is by parsing the denom, looking for a prefix that is only
// appended in the case of a bridged token. That is what this logic does.
//
// note that in the case of a refund, eg. when this function is called from `onTimeoutPacket`,
// the logic is inverted, as a prefix will only be prepended in the case the token is bridged in.
fn is_source(
    source_port: &PortId,
    source_channel: &ChannelId,
    denom: &Metadata,
    is_refund: bool,
) -> bool {
    let prefix = format!("{source_port}/{source_channel}/");

    if is_refund {
        !denom.starts_with(&prefix)
    } else {
        denom.starts_with(&prefix)
    }
}

#[derive(Clone)]
pub struct Ics20Transfer {}

#[async_trait]
pub trait Ics20TransferReadExt: StateRead {
    async fn withdrawal_check(
        &self,
        withdrawal: &Ics20Withdrawal,
        current_block_time: Time,
    ) -> Result<()> {
        // create packet
        let packet: IBCPacket<Unchecked> = withdrawal.clone().into();

        // send packet
        self.send_packet_check(packet, current_block_time).await?;

        Ok(())
    }
}

impl<T: StateRead + ?Sized> Ics20TransferReadExt for T {}

#[async_trait]
pub trait Ics20TransferWriteExt: StateWrite {
    async fn withdrawal_execute(&mut self, withdrawal: &Ics20Withdrawal) -> Result<()> {
        // create packet, assume it's already checked since the component caller contract calls `check` before `execute`
        let checked_packet = IBCPacket::<Unchecked>::from(withdrawal.clone()).assume_checked();

        let prefix = format!("transfer/{}/", &withdrawal.source_channel);
        if !withdrawal.denom.starts_with(&prefix) {
            // we are the source. add the value balance to the escrow channel.
            let existing_value_balance: Amount = self
                .get(&state_key::ics20_value_balance::by_asset_id(
                    &withdrawal.source_channel,
                    &withdrawal.denom.id(),
                ))
                .await
                .expect("able to retrieve value balance in ics20 withdrawal! (execute)")
                .unwrap_or_else(Amount::zero);

            let new_value_balance = existing_value_balance
                .checked_add(&withdrawal.amount)
                .ok_or_else(|| {
                    anyhow::anyhow!("overflow adding value balance in ics20 withdrawal")
                })?;
            self.put(
                state_key::ics20_value_balance::by_asset_id(
                    &withdrawal.source_channel,
                    &withdrawal.denom.id(),
                ),
                new_value_balance,
            );
            self.record_proto(
                event::EventOutboundFungibleTokenTransfer {
                    value: Value {
                        amount: withdrawal.amount,
                        asset_id: withdrawal.denom.id(),
                    },
                    sender: withdrawal.return_address.clone(),
                    receiver: withdrawal.destination_chain_address.clone(),
                    meta: FungibleTokenTransferPacketMetadata {
                        channel: withdrawal.source_channel.0.clone(),
                        sequence: self
                            .get_send_sequence(
                                &withdrawal.source_channel,
                                &checked_packet.source_port(),
                            )
                            .await?,
                    },
                }
                .to_proto(),
            );
        } else {
            // receiver is the source, burn utxos

            // double check the value balance here.
            //
            // for assets not originating from Penumbra, never transfer out more tokens than were
            // transferred in. (Our counterparties should be checking this anyways, since if we
            // were Byzantine we could lie to them).
            let value_balance: Amount = self
                .get(&state_key::ics20_value_balance::by_asset_id(
                    &withdrawal.source_channel,
                    &withdrawal.denom.id(),
                ))
                .await?
                .unwrap_or_else(Amount::zero);

            if value_balance < withdrawal.amount {
                anyhow::bail!("insufficient balance to withdraw tokens");
            }

            let new_value_balance =
                value_balance
                    .checked_sub(&withdrawal.amount)
                    .ok_or_else(|| {
                        anyhow::anyhow!("underflow subtracting value balance in ics20 withdrawal")
                    })?;
            self.put(
                state_key::ics20_value_balance::by_asset_id(
                    &withdrawal.source_channel,
                    &withdrawal.denom.id(),
                ),
                new_value_balance,
            );
            self.record_proto(
                event::EventOutboundFungibleTokenTransfer {
                    value: Value {
                        amount: withdrawal.amount,
                        asset_id: withdrawal.denom.id(),
                    },
                    sender: withdrawal.return_address.clone(),
                    receiver: withdrawal.destination_chain_address.clone(),
                    meta: FungibleTokenTransferPacketMetadata {
                        channel: withdrawal.source_channel.0.clone(),
                        sequence: self
                            .get_send_sequence(
                                &withdrawal.source_channel,
                                &checked_packet.source_port(),
                            )
                            .await?,
                    },
                }
                .to_proto(),
            );
        }

        self.send_packet_execute(checked_packet).await;

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> Ics20TransferWriteExt for T {}

// see: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer
#[async_trait]
impl AppHandlerCheck for Ics20Transfer {
    async fn chan_open_init_check<S: StateRead>(_state: S, msg: &MsgChannelOpenInit) -> Result<()> {
        if msg.ordering != ChannelOrder::Unordered {
            anyhow::bail!("channel order must be unordered for Ics20 transfer");
        }
        let ics20_version = Version::new("ics20-1".to_string());
        if msg.version_proposal != ics20_version {
            anyhow::bail!("channel version must be ics20 for Ics20 transfer");
        }

        Ok(())
    }

    async fn chan_open_try_check<S: StateRead>(_state: S, msg: &MsgChannelOpenTry) -> Result<()> {
        if msg.ordering != ChannelOrder::Unordered {
            anyhow::bail!("channel order must be unordered for Ics20 transfer");
        }
        let ics20_version = Version::new("ics20-1".to_string());

        if msg.version_supported_on_a != ics20_version {
            anyhow::bail!("counterparty version must be ics20-1 for Ics20 transfer");
        }

        Ok(())
    }

    async fn chan_open_ack_check<S: StateRead>(_state: S, msg: &MsgChannelOpenAck) -> Result<()> {
        let ics20_version = Version::new("ics20-1".to_string());
        if msg.version_on_b != ics20_version {
            anyhow::bail!("counterparty version must be ics20-1 for Ics20 transfer");
        }

        Ok(())
    }

    async fn chan_open_confirm_check<S: StateRead>(
        _state: S,
        _msg: &MsgChannelOpenConfirm,
    ) -> Result<()> {
        // accept channel confirmations, port has already been validated, version has already been validated
        Ok(())
    }

    async fn chan_close_confirm_check<S: StateRead>(
        _state: S,
        _msg: &MsgChannelCloseConfirm,
    ) -> Result<()> {
        // no action necessary
        Ok(())
    }

    async fn chan_close_init_check<S: StateRead>(
        _state: S,
        _msg: &MsgChannelCloseInit,
    ) -> Result<()> {
        // always abort transaction
        anyhow::bail!("ics20 always aborts on close init");
    }

    async fn recv_packet_check<S: StateRead>(_state: S, _msg: &MsgRecvPacket) -> Result<()> {
        // all checks on recv_packet done in execute
        Ok(())
    }

    async fn timeout_packet_check<S: StateRead>(state: S, msg: &MsgTimeout) -> Result<()> {
        let packet_data: FungibleTokenPacketData =
            serde_json::from_slice(msg.packet.data.as_slice())?;
        let denom: asset::Metadata = packet_data.denom.as_str().try_into()?;

        if is_source(&msg.packet.port_on_a, &msg.packet.chan_on_a, &denom, true) {
            // check if we have enough balance to refund tokens to sender
            let value_balance: Amount = state
                .get(&state_key::ics20_value_balance::by_asset_id(
                    &msg.packet.chan_on_a,
                    &denom.id(),
                ))
                .await?
                .unwrap_or_else(Amount::zero);

            let amount_penumbra: Amount = packet_data.amount.try_into()?;
            if value_balance < amount_penumbra {
                anyhow::bail!("insufficient balance to refund tokens to sender");
            }
        }

        Ok(())
    }

    async fn acknowledge_packet_check<S: StateRead>(
        _state: S,
        _msg: &MsgAcknowledgement,
    ) -> Result<()> {
        Ok(())
    }
}

// the main entry point for ICS20 transfer packet handling
async fn recv_transfer_packet_inner<S: StateWrite>(
    mut state: S,
    msg: &MsgRecvPacket,
) -> Result<()> {
    // parse if we are source or dest, and mint or burn accordingly
    //
    // see this part of the spec for this logic:
    //
    // https://github.com/cosmos/ibc/tree/main/spec/app/ics-020-fungible-token-transfer (onRecvPacket)
    //
    // NOTE: spec says proto but this is actually JSON according to the ibc-go implementation
    let packet_data: FungibleTokenPacketData = serde_json::from_slice(msg.packet.data.as_slice())
        .with_context(|| "failed to decode FTPD packet")?;
    let packet_denom: asset::Metadata = packet_data
        .denom
        .as_str()
        .try_into()
        .context("couldnt decode denom in ICS20 transfer")?;
    let receiver_amount: Amount = packet_data
        .amount
        .try_into()
        .context("couldnt decode amount in ICS20 transfer")?;
    let receiver_address = Address::from_str(&packet_data.receiver)?;

    // NOTE: here we assume we are chain A.

    // 2. check if we are the source chain for the denom.
    if is_source(
        &msg.packet.port_on_a,
        &msg.packet.chan_on_a,
        &packet_denom,
        false,
    ) {
        // mint tokens to receiver in the amount of packet_data.amount in the denom of denom (with
        // the source removed, since we're the source)
        let prefix = format!(
            "{source_port}/{source_chan}/",
            source_port = msg.packet.port_on_a,
            source_chan = msg.packet.chan_on_a
        );

        let denom: asset::Metadata = packet_data
            .denom
            .strip_prefix(&prefix)
            .context(format!(
                "denom in packet didn't begin with expected prefix {}",
                prefix
            ))?
            .try_into()
            .context("couldnt decode denom in ICS20 transfer")?;

        let value: Value = Value {
            amount: receiver_amount,
            asset_id: denom.id(),
        };

        // assume AppHandlerCheck has already been called, and we have enough balance to mint tokens to receiver
        // check if we have enough balance to unescrow tokens to receiver
        let value_balance: Amount = state
            .get(&state_key::ics20_value_balance::by_asset_id(
                &msg.packet.chan_on_b,
                &denom.id(),
            ))
            .await?
            .unwrap_or_else(Amount::zero);

        if value_balance < receiver_amount {
            // error text here is from the ics20 spec
            anyhow::bail!("transfer coins failed");
        }

        state
            .mint_note(
                value,
                &receiver_address,
                CommitmentSource::Ics20Transfer {
                    packet_seq: msg.packet.sequence.0,
                    // We are chain A
                    channel_id: msg.packet.chan_on_a.0.clone(),
                    sender: packet_data.sender.clone(),
                },
            )
            .await
            .context("unable to mint note when receiving ics20 transfer packet")?;

        // update the value balance
        // note: this arithmetic was checked above, but we do it again anyway.
        let new_value_balance = value_balance
            .checked_sub(&receiver_amount)
            .context("underflow subtracing value balance in ics20 transfer")?;
        state.put(
            state_key::ics20_value_balance::by_asset_id(&msg.packet.chan_on_b, &denom.id()),
            new_value_balance,
        );
        state.record_proto(
            event::EventInboundFungibleTokenTransfer {
                value,
                sender: packet_data.sender.clone(),
                receiver: receiver_address,
                meta: FungibleTokenTransferPacketMetadata {
                    channel: msg.packet.chan_on_a.0.clone(),
                    sequence: msg.packet.sequence.0,
                },
            }
            .to_proto(),
        );
    } else {
        // create new denom:
        //
        // prefix = "{packet.destPort}/{packet.destChannel}/"
        // prefixedDenomination = prefix + data.denom
        //
        // then mint that denom to packet_data.receiver in packet_data.amount
        let prefixed_denomination = format!(
            "{}/{}/{}",
            msg.packet.port_on_b, msg.packet.chan_on_b, packet_data.denom
        );

        let denom: asset::Metadata = prefixed_denomination
            .as_str()
            .try_into()
            .context("unable to parse denom in ics20 transfer as DenomMetadata")?;
        state.register_denom(&denom).await;

        let value = Value {
            amount: receiver_amount,
            asset_id: denom.id(),
        };

        state
            .mint_note(
                value,
                &receiver_address,
                CommitmentSource::Ics20Transfer {
                    packet_seq: msg.packet.sequence.0,
                    // We are chain A
                    channel_id: msg.packet.chan_on_a.0.clone(),
                    sender: packet_data.sender.clone(),
                },
            )
            .await
            .context("failed to mint notes in ibc transfer")?;

        // update the value balance
        let value_balance: Amount = state
            .get(&state_key::ics20_value_balance::by_asset_id(
                &msg.packet.chan_on_b,
                &denom.id(),
            ))
            .await?
            .unwrap_or_else(Amount::zero);

        let new_value_balance = value_balance.saturating_add(&value.amount);
        state.put(
            state_key::ics20_value_balance::by_asset_id(&msg.packet.chan_on_b, &denom.id()),
            new_value_balance,
        );
        state.record_proto(
            event::EventInboundFungibleTokenTransfer {
                value,
                sender: packet_data.sender.clone(),
                receiver: receiver_address,
                meta: FungibleTokenTransferPacketMetadata {
                    channel: msg.packet.chan_on_a.0.clone(),
                    sequence: msg.packet.sequence.0,
                },
            }
            .to_proto(),
        );
    }

    Ok(())
}

// see: https://github.com/cosmos/ibc/blob/8326e26e7e1188b95c32481ff00348a705b23700/spec/app/ics-020-fungible-token-transfer/README.md?plain=1#L297
async fn refund_tokens<S: StateWrite>(
    mut state: S,
    packet: &Packet,
    reason: event::FungibleTokenRefundReason,
) -> Result<()> {
    let packet_data: FungibleTokenPacketData = serde_json::from_slice(packet.data.as_slice())?;
    let denom: asset::Metadata = packet_data // CRITICAL: verify that this denom is validated in upstream timeout handling
        .denom
        .as_str()
        .try_into()
        .context("couldn't decode denom in ics20 transfer timeout")?;
    // receiver was source chain, mint vouchers back to sender
    let amount: Amount = packet_data
        .amount
        .try_into()
        .context("couldn't decode amount in ics20 transfer timeout")?;

    // packet_data.sender is the original sender for this packet that was not committed on the
    // other chain but was sent from penumbra. so, the penumbra refund receiver address is the
    // sender
    let receiver = Address::from_str(&packet_data.sender)
        .context("couldn't decode receiver address in ics20 timeout")?;

    let value: Value = Value {
        amount,
        asset_id: denom.id(),
    };

    if is_source(&packet.port_on_a, &packet.chan_on_a, &denom, true) {
        // sender was source chain, unescrow tokens back to sender
        let value_balance: Amount = state
            .get(&state_key::ics20_value_balance::by_asset_id(
                &packet.chan_on_a,
                &denom.id(),
            ))
            .await?
            .unwrap_or_else(Amount::zero);

        if value_balance < amount {
            anyhow::bail!("couldn't return coins in timeout: not enough value balance");
        }

        state
            .mint_note(
                value,
                &receiver,
                CommitmentSource::Ics20Transfer {
                    packet_seq: packet.sequence.0,
                    channel_id: packet.chan_on_a.0.clone(),
                    sender: packet_data.sender.clone(),
                },
            )
            .await
            .context("couldn't mint note in timeout_packet_inner")?;

        // update the value balance
        // note: this arithmetic was checked above, but we do it again anyway.
        let new_value_balance = value_balance
            .checked_sub(&amount)
            .context("underflow in ics20 timeout packet value balance subtraction")?;
        state.put(
            state_key::ics20_value_balance::by_asset_id(&packet.chan_on_a, &denom.id()),
            new_value_balance,
        );
        state.record_proto(
            event::EventOutboundFungibleTokenRefund {
                value,
                sender: receiver, // note, this comes from packet_data.sender
                receiver: packet_data.receiver.clone(),
                reason,
                // Use the destination channel, i.e. our name for it, to be consistent across events.
                meta: FungibleTokenTransferPacketMetadata {
                    channel: packet.chan_on_b.0.clone(),
                    sequence: packet.sequence.0,
                },
            }
            .to_proto(),
        );
    } else {
        let value_balance: Amount = state
            .get(&state_key::ics20_value_balance::by_asset_id(
                &packet.chan_on_a,
                &denom.id(),
            ))
            .await?
            .unwrap_or_else(Amount::zero);

        state
            .mint_note(
                value,
                &receiver,
                // NOTE: should this be Ics20TransferTimeout?
                CommitmentSource::Ics20Transfer {
                    packet_seq: packet.sequence.0,
                    channel_id: packet.chan_on_a.0.clone(),
                    sender: packet_data.sender.clone(),
                },
            )
            .await
            .context("failed to mint return voucher in ics20 transfer timeout")?;

        let new_value_balance = value_balance.saturating_add(&value.amount);
        state.put(
            state_key::ics20_value_balance::by_asset_id(&packet.chan_on_a, &denom.id()),
            new_value_balance,
        );
        state.record_proto(
            event::EventOutboundFungibleTokenRefund {
                value,
                sender: receiver, // note, this comes from packet_data.sender
                receiver: packet_data.receiver.clone(),
                reason,
                // Use the destination channel, i.e. our name for it, to be consistent across events.
                meta: FungibleTokenTransferPacketMetadata {
                    channel: packet.chan_on_b.0.clone(),
                    sequence: packet.sequence.0,
                },
            }
            .to_proto(),
        );
    }

    Ok(())
}

// NOTE: should these be fallible, now that our enclosing state machine is fallible in execution?
#[async_trait]
impl AppHandlerExecute for Ics20Transfer {
    async fn chan_open_init_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenInit) {}
    async fn chan_open_try_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenTry) {}
    async fn chan_open_ack_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenAck) {}
    async fn chan_open_confirm_execute<S: StateWrite>(_state: S, _msg: &MsgChannelOpenConfirm) {}
    async fn chan_close_confirm_execute<S: StateWrite>(_state: S, _msg: &MsgChannelCloseConfirm) {}
    async fn chan_close_init_execute<S: StateWrite>(_state: S, _msg: &MsgChannelCloseInit) {}
    async fn recv_packet_execute<S: StateWrite>(mut state: S, msg: &MsgRecvPacket) -> Result<()> {
        // recv packet should never fail a transaction, but it should record a failure acknowledgement.
        let ack: Vec<u8> = match recv_transfer_packet_inner(&mut state, msg).await {
            Ok(_) => {
                // record packet acknowledgement without error
                TokenTransferAcknowledgement::success().into()
            }
            Err(e) => {
                tracing::debug!("couldnt execute transfer: {:#}", e);
                // record packet acknowledgement with error
                TokenTransferAcknowledgement::Error(e.to_string()).into()
            }
        };

        state
            .write_acknowledgement(&msg.packet, &ack)
            .await
            .context("able to write acknowledgement")?;

        Ok(())
    }

    async fn timeout_packet_execute<S: StateWrite>(mut state: S, msg: &MsgTimeout) -> Result<()> {
        // timeouts may fail due to counterparty chains sending transfers of u128-1
        refund_tokens(
            &mut state,
            &msg.packet,
            event::FungibleTokenRefundReason::Timeout,
        )
        .await
        .context("able to timeout packet")?;

        Ok(())
    }

    async fn acknowledge_packet_execute<S: StateWrite>(
        mut state: S,
        msg: &MsgAcknowledgement,
    ) -> Result<()> {
        let ack: TokenTransferAcknowledgement =
            serde_json::from_slice(msg.acknowledgement.as_slice())?;
        if !ack.is_successful() {
            // in the case where a counterparty chain acknowledges a packet with an error,
            // for example due to a middleware processing issue or other behavior,
            // the funds should be unescrowed back to the packet sender.
            refund_tokens(
                &mut state,
                &msg.packet,
                event::FungibleTokenRefundReason::Error,
            )
            .await
            .context("unable to refund packet acknowledgement")?;
        }

        Ok(())
    }
}

impl AppHandler for Ics20Transfer {}
