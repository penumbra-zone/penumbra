impl serde::Serialize for AssetInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.asset_id.is_some() {
            len += 1;
        }
        if self.denom.is_some() {
            len += 1;
        }
        if self.as_of_block_height != 0 {
            len += 1;
        }
        if self.total_supply != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.AssetInfo", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.denom.as_ref() {
            struct_ser.serialize_field("denom", v)?;
        }
        if self.as_of_block_height != 0 {
            struct_ser.serialize_field("asOfBlockHeight", ToString::to_string(&self.as_of_block_height).as_str())?;
        }
        if self.total_supply != 0 {
            struct_ser.serialize_field("totalSupply", ToString::to_string(&self.total_supply).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AssetInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "denom",
            "as_of_block_height",
            "asOfBlockHeight",
            "total_supply",
            "totalSupply",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            Denom,
            AsOfBlockHeight,
            TotalSupply,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "assetId" | "asset_id" => Ok(GeneratedField::AssetId),
                            "denom" => Ok(GeneratedField::Denom),
                            "asOfBlockHeight" | "as_of_block_height" => Ok(GeneratedField::AsOfBlockHeight),
                            "totalSupply" | "total_supply" => Ok(GeneratedField::TotalSupply),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AssetInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.AssetInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AssetInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut denom__ = None;
                let mut as_of_block_height__ = None;
                let mut total_supply__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map.next_value()?;
                        }
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = map.next_value()?;
                        }
                        GeneratedField::AsOfBlockHeight => {
                            if as_of_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfBlockHeight"));
                            }
                            as_of_block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TotalSupply => {
                            if total_supply__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalSupply"));
                            }
                            total_supply__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(AssetInfo {
                    asset_id: asset_id__,
                    denom: denom__,
                    as_of_block_height: as_of_block_height__.unwrap_or_default(),
                    total_supply: total_supply__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.AssetInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChainParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.epoch_duration != 0 {
            len += 1;
        }
        if self.unbonding_epochs != 0 {
            len += 1;
        }
        if self.active_validator_limit != 0 {
            len += 1;
        }
        if self.base_reward_rate != 0 {
            len += 1;
        }
        if self.slashing_penalty_misbehavior.is_some() {
            len += 1;
        }
        if self.slashing_penalty_downtime.is_some() {
            len += 1;
        }
        if self.signed_blocks_window_len != 0 {
            len += 1;
        }
        if self.missed_blocks_maximum != 0 {
            len += 1;
        }
        if self.ibc_enabled {
            len += 1;
        }
        if self.inbound_ics20_transfers_enabled {
            len += 1;
        }
        if self.outbound_ics20_transfers_enabled {
            len += 1;
        }
        if self.proposal_voting_blocks != 0 {
            len += 1;
        }
        if self.proposal_deposit_amount.is_some() {
            len += 1;
        }
        if self.proposal_valid_quorum.is_some() {
            len += 1;
        }
        if self.proposal_pass_threshold.is_some() {
            len += 1;
        }
        if self.proposal_slash_threshold.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.ChainParameters", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if self.epoch_duration != 0 {
            struct_ser.serialize_field("epochDuration", ToString::to_string(&self.epoch_duration).as_str())?;
        }
        if self.unbonding_epochs != 0 {
            struct_ser.serialize_field("unbondingEpochs", ToString::to_string(&self.unbonding_epochs).as_str())?;
        }
        if self.active_validator_limit != 0 {
            struct_ser.serialize_field("activeValidatorLimit", ToString::to_string(&self.active_validator_limit).as_str())?;
        }
        if self.base_reward_rate != 0 {
            struct_ser.serialize_field("baseRewardRate", ToString::to_string(&self.base_reward_rate).as_str())?;
        }
        if let Some(v) = self.slashing_penalty_misbehavior.as_ref() {
            struct_ser.serialize_field("slashingPenaltyMisbehavior", v)?;
        }
        if let Some(v) = self.slashing_penalty_downtime.as_ref() {
            struct_ser.serialize_field("slashingPenaltyDowntime", v)?;
        }
        if self.signed_blocks_window_len != 0 {
            struct_ser.serialize_field("signedBlocksWindowLen", ToString::to_string(&self.signed_blocks_window_len).as_str())?;
        }
        if self.missed_blocks_maximum != 0 {
            struct_ser.serialize_field("missedBlocksMaximum", ToString::to_string(&self.missed_blocks_maximum).as_str())?;
        }
        if self.ibc_enabled {
            struct_ser.serialize_field("ibcEnabled", &self.ibc_enabled)?;
        }
        if self.inbound_ics20_transfers_enabled {
            struct_ser.serialize_field("inboundIcs20TransfersEnabled", &self.inbound_ics20_transfers_enabled)?;
        }
        if self.outbound_ics20_transfers_enabled {
            struct_ser.serialize_field("outboundIcs20TransfersEnabled", &self.outbound_ics20_transfers_enabled)?;
        }
        if self.proposal_voting_blocks != 0 {
            struct_ser.serialize_field("proposalVotingBlocks", ToString::to_string(&self.proposal_voting_blocks).as_str())?;
        }
        if let Some(v) = self.proposal_deposit_amount.as_ref() {
            struct_ser.serialize_field("proposalDepositAmount", v)?;
        }
        if let Some(v) = self.proposal_valid_quorum.as_ref() {
            struct_ser.serialize_field("proposalValidQuorum", v)?;
        }
        if let Some(v) = self.proposal_pass_threshold.as_ref() {
            struct_ser.serialize_field("proposalPassThreshold", v)?;
        }
        if let Some(v) = self.proposal_slash_threshold.as_ref() {
            struct_ser.serialize_field("proposalSlashThreshold", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChainParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "epoch_duration",
            "epochDuration",
            "unbonding_epochs",
            "unbondingEpochs",
            "active_validator_limit",
            "activeValidatorLimit",
            "base_reward_rate",
            "baseRewardRate",
            "slashing_penalty_misbehavior",
            "slashingPenaltyMisbehavior",
            "slashing_penalty_downtime",
            "slashingPenaltyDowntime",
            "signed_blocks_window_len",
            "signedBlocksWindowLen",
            "missed_blocks_maximum",
            "missedBlocksMaximum",
            "ibc_enabled",
            "ibcEnabled",
            "inbound_ics20_transfers_enabled",
            "inboundIcs20TransfersEnabled",
            "outbound_ics20_transfers_enabled",
            "outboundIcs20TransfersEnabled",
            "proposal_voting_blocks",
            "proposalVotingBlocks",
            "proposal_deposit_amount",
            "proposalDepositAmount",
            "proposal_valid_quorum",
            "proposalValidQuorum",
            "proposal_pass_threshold",
            "proposalPassThreshold",
            "proposal_slash_threshold",
            "proposalSlashThreshold",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            EpochDuration,
            UnbondingEpochs,
            ActiveValidatorLimit,
            BaseRewardRate,
            SlashingPenaltyMisbehavior,
            SlashingPenaltyDowntime,
            SignedBlocksWindowLen,
            MissedBlocksMaximum,
            IbcEnabled,
            InboundIcs20TransfersEnabled,
            OutboundIcs20TransfersEnabled,
            ProposalVotingBlocks,
            ProposalDepositAmount,
            ProposalValidQuorum,
            ProposalPassThreshold,
            ProposalSlashThreshold,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "epochDuration" | "epoch_duration" => Ok(GeneratedField::EpochDuration),
                            "unbondingEpochs" | "unbonding_epochs" => Ok(GeneratedField::UnbondingEpochs),
                            "activeValidatorLimit" | "active_validator_limit" => Ok(GeneratedField::ActiveValidatorLimit),
                            "baseRewardRate" | "base_reward_rate" => Ok(GeneratedField::BaseRewardRate),
                            "slashingPenaltyMisbehavior" | "slashing_penalty_misbehavior" => Ok(GeneratedField::SlashingPenaltyMisbehavior),
                            "slashingPenaltyDowntime" | "slashing_penalty_downtime" => Ok(GeneratedField::SlashingPenaltyDowntime),
                            "signedBlocksWindowLen" | "signed_blocks_window_len" => Ok(GeneratedField::SignedBlocksWindowLen),
                            "missedBlocksMaximum" | "missed_blocks_maximum" => Ok(GeneratedField::MissedBlocksMaximum),
                            "ibcEnabled" | "ibc_enabled" => Ok(GeneratedField::IbcEnabled),
                            "inboundIcs20TransfersEnabled" | "inbound_ics20_transfers_enabled" => Ok(GeneratedField::InboundIcs20TransfersEnabled),
                            "outboundIcs20TransfersEnabled" | "outbound_ics20_transfers_enabled" => Ok(GeneratedField::OutboundIcs20TransfersEnabled),
                            "proposalVotingBlocks" | "proposal_voting_blocks" => Ok(GeneratedField::ProposalVotingBlocks),
                            "proposalDepositAmount" | "proposal_deposit_amount" => Ok(GeneratedField::ProposalDepositAmount),
                            "proposalValidQuorum" | "proposal_valid_quorum" => Ok(GeneratedField::ProposalValidQuorum),
                            "proposalPassThreshold" | "proposal_pass_threshold" => Ok(GeneratedField::ProposalPassThreshold),
                            "proposalSlashThreshold" | "proposal_slash_threshold" => Ok(GeneratedField::ProposalSlashThreshold),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChainParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.ChainParameters")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChainParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut epoch_duration__ = None;
                let mut unbonding_epochs__ = None;
                let mut active_validator_limit__ = None;
                let mut base_reward_rate__ = None;
                let mut slashing_penalty_misbehavior__ = None;
                let mut slashing_penalty_downtime__ = None;
                let mut signed_blocks_window_len__ = None;
                let mut missed_blocks_maximum__ = None;
                let mut ibc_enabled__ = None;
                let mut inbound_ics20_transfers_enabled__ = None;
                let mut outbound_ics20_transfers_enabled__ = None;
                let mut proposal_voting_blocks__ = None;
                let mut proposal_deposit_amount__ = None;
                let mut proposal_valid_quorum__ = None;
                let mut proposal_pass_threshold__ = None;
                let mut proposal_slash_threshold__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::EpochDuration => {
                            if epoch_duration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochDuration"));
                            }
                            epoch_duration__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UnbondingEpochs => {
                            if unbonding_epochs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingEpochs"));
                            }
                            unbonding_epochs__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ActiveValidatorLimit => {
                            if active_validator_limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("activeValidatorLimit"));
                            }
                            active_validator_limit__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::BaseRewardRate => {
                            if base_reward_rate__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseRewardRate"));
                            }
                            base_reward_rate__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::SlashingPenaltyMisbehavior => {
                            if slashing_penalty_misbehavior__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashingPenaltyMisbehavior"));
                            }
                            slashing_penalty_misbehavior__ = map.next_value()?;
                        }
                        GeneratedField::SlashingPenaltyDowntime => {
                            if slashing_penalty_downtime__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashingPenaltyDowntime"));
                            }
                            slashing_penalty_downtime__ = map.next_value()?;
                        }
                        GeneratedField::SignedBlocksWindowLen => {
                            if signed_blocks_window_len__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedBlocksWindowLen"));
                            }
                            signed_blocks_window_len__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MissedBlocksMaximum => {
                            if missed_blocks_maximum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("missedBlocksMaximum"));
                            }
                            missed_blocks_maximum__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::IbcEnabled => {
                            if ibc_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcEnabled"));
                            }
                            ibc_enabled__ = Some(map.next_value()?);
                        }
                        GeneratedField::InboundIcs20TransfersEnabled => {
                            if inbound_ics20_transfers_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inboundIcs20TransfersEnabled"));
                            }
                            inbound_ics20_transfers_enabled__ = Some(map.next_value()?);
                        }
                        GeneratedField::OutboundIcs20TransfersEnabled => {
                            if outbound_ics20_transfers_enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outboundIcs20TransfersEnabled"));
                            }
                            outbound_ics20_transfers_enabled__ = Some(map.next_value()?);
                        }
                        GeneratedField::ProposalVotingBlocks => {
                            if proposal_voting_blocks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalVotingBlocks"));
                            }
                            proposal_voting_blocks__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProposalDepositAmount => {
                            if proposal_deposit_amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositAmount"));
                            }
                            proposal_deposit_amount__ = map.next_value()?;
                        }
                        GeneratedField::ProposalValidQuorum => {
                            if proposal_valid_quorum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalValidQuorum"));
                            }
                            proposal_valid_quorum__ = map.next_value()?;
                        }
                        GeneratedField::ProposalPassThreshold => {
                            if proposal_pass_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalPassThreshold"));
                            }
                            proposal_pass_threshold__ = map.next_value()?;
                        }
                        GeneratedField::ProposalSlashThreshold => {
                            if proposal_slash_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSlashThreshold"));
                            }
                            proposal_slash_threshold__ = map.next_value()?;
                        }
                    }
                }
                Ok(ChainParameters {
                    chain_id: chain_id__.unwrap_or_default(),
                    epoch_duration: epoch_duration__.unwrap_or_default(),
                    unbonding_epochs: unbonding_epochs__.unwrap_or_default(),
                    active_validator_limit: active_validator_limit__.unwrap_or_default(),
                    base_reward_rate: base_reward_rate__.unwrap_or_default(),
                    slashing_penalty_misbehavior: slashing_penalty_misbehavior__,
                    slashing_penalty_downtime: slashing_penalty_downtime__,
                    signed_blocks_window_len: signed_blocks_window_len__.unwrap_or_default(),
                    missed_blocks_maximum: missed_blocks_maximum__.unwrap_or_default(),
                    ibc_enabled: ibc_enabled__.unwrap_or_default(),
                    inbound_ics20_transfers_enabled: inbound_ics20_transfers_enabled__.unwrap_or_default(),
                    outbound_ics20_transfers_enabled: outbound_ics20_transfers_enabled__.unwrap_or_default(),
                    proposal_voting_blocks: proposal_voting_blocks__.unwrap_or_default(),
                    proposal_deposit_amount: proposal_deposit_amount__,
                    proposal_valid_quorum: proposal_valid_quorum__,
                    proposal_pass_threshold: proposal_pass_threshold__,
                    proposal_slash_threshold: proposal_slash_threshold__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.ChainParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CompactBlock {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.height != 0 {
            len += 1;
        }
        if !self.state_payloads.is_empty() {
            len += 1;
        }
        if !self.nullifiers.is_empty() {
            len += 1;
        }
        if self.block_root.is_some() {
            len += 1;
        }
        if self.epoch_root.is_some() {
            len += 1;
        }
        if self.proposal_started {
            len += 1;
        }
        if self.fmd_parameters.is_some() {
            len += 1;
        }
        if !self.swap_outputs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.CompactBlock", len)?;
        if self.height != 0 {
            struct_ser.serialize_field("height", ToString::to_string(&self.height).as_str())?;
        }
        if !self.state_payloads.is_empty() {
            struct_ser.serialize_field("statePayloads", &self.state_payloads)?;
        }
        if !self.nullifiers.is_empty() {
            struct_ser.serialize_field("nullifiers", &self.nullifiers)?;
        }
        if let Some(v) = self.block_root.as_ref() {
            struct_ser.serialize_field("blockRoot", v)?;
        }
        if let Some(v) = self.epoch_root.as_ref() {
            struct_ser.serialize_field("epochRoot", v)?;
        }
        if self.proposal_started {
            struct_ser.serialize_field("proposalStarted", &self.proposal_started)?;
        }
        if let Some(v) = self.fmd_parameters.as_ref() {
            struct_ser.serialize_field("fmdParameters", v)?;
        }
        if !self.swap_outputs.is_empty() {
            struct_ser.serialize_field("swapOutputs", &self.swap_outputs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CompactBlock {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "height",
            "state_payloads",
            "statePayloads",
            "nullifiers",
            "block_root",
            "blockRoot",
            "epoch_root",
            "epochRoot",
            "proposal_started",
            "proposalStarted",
            "fmd_parameters",
            "fmdParameters",
            "swap_outputs",
            "swapOutputs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Height,
            StatePayloads,
            Nullifiers,
            BlockRoot,
            EpochRoot,
            ProposalStarted,
            FmdParameters,
            SwapOutputs,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "height" => Ok(GeneratedField::Height),
                            "statePayloads" | "state_payloads" => Ok(GeneratedField::StatePayloads),
                            "nullifiers" => Ok(GeneratedField::Nullifiers),
                            "blockRoot" | "block_root" => Ok(GeneratedField::BlockRoot),
                            "epochRoot" | "epoch_root" => Ok(GeneratedField::EpochRoot),
                            "proposalStarted" | "proposal_started" => Ok(GeneratedField::ProposalStarted),
                            "fmdParameters" | "fmd_parameters" => Ok(GeneratedField::FmdParameters),
                            "swapOutputs" | "swap_outputs" => Ok(GeneratedField::SwapOutputs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CompactBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.CompactBlock")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CompactBlock, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut height__ = None;
                let mut state_payloads__ = None;
                let mut nullifiers__ = None;
                let mut block_root__ = None;
                let mut epoch_root__ = None;
                let mut proposal_started__ = None;
                let mut fmd_parameters__ = None;
                let mut swap_outputs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Height => {
                            if height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("height"));
                            }
                            height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::StatePayloads => {
                            if state_payloads__.is_some() {
                                return Err(serde::de::Error::duplicate_field("statePayloads"));
                            }
                            state_payloads__ = Some(map.next_value()?);
                        }
                        GeneratedField::Nullifiers => {
                            if nullifiers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifiers"));
                            }
                            nullifiers__ = Some(map.next_value()?);
                        }
                        GeneratedField::BlockRoot => {
                            if block_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("blockRoot"));
                            }
                            block_root__ = map.next_value()?;
                        }
                        GeneratedField::EpochRoot => {
                            if epoch_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("epochRoot"));
                            }
                            epoch_root__ = map.next_value()?;
                        }
                        GeneratedField::ProposalStarted => {
                            if proposal_started__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalStarted"));
                            }
                            proposal_started__ = Some(map.next_value()?);
                        }
                        GeneratedField::FmdParameters => {
                            if fmd_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdParameters"));
                            }
                            fmd_parameters__ = map.next_value()?;
                        }
                        GeneratedField::SwapOutputs => {
                            if swap_outputs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapOutputs"));
                            }
                            swap_outputs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CompactBlock {
                    height: height__.unwrap_or_default(),
                    state_payloads: state_payloads__.unwrap_or_default(),
                    nullifiers: nullifiers__.unwrap_or_default(),
                    block_root: block_root__,
                    epoch_root: epoch_root__,
                    proposal_started: proposal_started__.unwrap_or_default(),
                    fmd_parameters: fmd_parameters__,
                    swap_outputs: swap_outputs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.CompactBlock", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FmdParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.precision_bits != 0 {
            len += 1;
        }
        if self.as_of_block_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.FmdParameters", len)?;
        if self.precision_bits != 0 {
            struct_ser.serialize_field("precisionBits", &self.precision_bits)?;
        }
        if self.as_of_block_height != 0 {
            struct_ser.serialize_field("asOfBlockHeight", ToString::to_string(&self.as_of_block_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FmdParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "precision_bits",
            "precisionBits",
            "as_of_block_height",
            "asOfBlockHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrecisionBits,
            AsOfBlockHeight,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "precisionBits" | "precision_bits" => Ok(GeneratedField::PrecisionBits),
                            "asOfBlockHeight" | "as_of_block_height" => Ok(GeneratedField::AsOfBlockHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FmdParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.FmdParameters")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FmdParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut precision_bits__ = None;
                let mut as_of_block_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PrecisionBits => {
                            if precision_bits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("precisionBits"));
                            }
                            precision_bits__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AsOfBlockHeight => {
                            if as_of_block_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("asOfBlockHeight"));
                            }
                            as_of_block_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(FmdParameters {
                    precision_bits: precision_bits__.unwrap_or_default(),
                    as_of_block_height: as_of_block_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.FmdParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisAppState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.chain_params.is_some() {
            len += 1;
        }
        if !self.validators.is_empty() {
            len += 1;
        }
        if !self.allocations.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.GenesisAppState", len)?;
        if let Some(v) = self.chain_params.as_ref() {
            struct_ser.serialize_field("chainParams", v)?;
        }
        if !self.validators.is_empty() {
            struct_ser.serialize_field("validators", &self.validators)?;
        }
        if !self.allocations.is_empty() {
            struct_ser.serialize_field("allocations", &self.allocations)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisAppState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_params",
            "chainParams",
            "validators",
            "allocations",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainParams,
            Validators,
            Allocations,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "chainParams" | "chain_params" => Ok(GeneratedField::ChainParams),
                            "validators" => Ok(GeneratedField::Validators),
                            "allocations" => Ok(GeneratedField::Allocations),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisAppState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.GenesisAppState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisAppState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_params__ = None;
                let mut validators__ = None;
                let mut allocations__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainParams => {
                            if chain_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainParams"));
                            }
                            chain_params__ = map.next_value()?;
                        }
                        GeneratedField::Validators => {
                            if validators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validators"));
                            }
                            validators__ = Some(map.next_value()?);
                        }
                        GeneratedField::Allocations => {
                            if allocations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allocations"));
                            }
                            allocations__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisAppState {
                    chain_params: chain_params__,
                    validators: validators__.unwrap_or_default(),
                    allocations: allocations__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.GenesisAppState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for genesis_app_state::Allocation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.amount != 0 {
            len += 1;
        }
        if !self.denom.is_empty() {
            len += 1;
        }
        if self.address.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.GenesisAppState.Allocation", len)?;
        if self.amount != 0 {
            struct_ser.serialize_field("amount", ToString::to_string(&self.amount).as_str())?;
        }
        if !self.denom.is_empty() {
            struct_ser.serialize_field("denom", &self.denom)?;
        }
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for genesis_app_state::Allocation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "amount",
            "denom",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Amount,
            Denom,
            Address,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "amount" => Ok(GeneratedField::Amount),
                            "denom" => Ok(GeneratedField::Denom),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = genesis_app_state::Allocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.GenesisAppState.Allocation")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<genesis_app_state::Allocation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut amount__ = None;
                let mut denom__ = None;
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Amount => {
                            if amount__.is_some() {
                                return Err(serde::de::Error::duplicate_field("amount"));
                            }
                            amount__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Denom => {
                            if denom__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denom"));
                            }
                            denom__ = Some(map.next_value()?);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                    }
                }
                Ok(genesis_app_state::Allocation {
                    amount: amount__.unwrap_or_default(),
                    denom: denom__.unwrap_or_default(),
                    address: address__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.GenesisAppState.Allocation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for KnownAssets {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.assets.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.KnownAssets", len)?;
        if !self.assets.is_empty() {
            struct_ser.serialize_field("assets", &self.assets)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for KnownAssets {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "assets",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Assets,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "assets" => Ok(GeneratedField::Assets),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = KnownAssets;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.KnownAssets")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<KnownAssets, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut assets__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Assets => {
                            if assets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assets"));
                            }
                            assets__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(KnownAssets {
                    assets: assets__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.KnownAssets", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NoteSource {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.inner.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.NoteSource", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NoteSource {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "inner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Inner,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "inner" => Ok(GeneratedField::Inner),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NoteSource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.NoteSource")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NoteSource, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(NoteSource {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.NoteSource", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Ratio {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.numerator != 0 {
            len += 1;
        }
        if self.denominator != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.Ratio", len)?;
        if self.numerator != 0 {
            struct_ser.serialize_field("numerator", ToString::to_string(&self.numerator).as_str())?;
        }
        if self.denominator != 0 {
            struct_ser.serialize_field("denominator", ToString::to_string(&self.denominator).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Ratio {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "numerator",
            "denominator",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Numerator,
            Denominator,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "numerator" => Ok(GeneratedField::Numerator),
                            "denominator" => Ok(GeneratedField::Denominator),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Ratio;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.Ratio")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Ratio, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut numerator__ = None;
                let mut denominator__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Numerator => {
                            if numerator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numerator"));
                            }
                            numerator__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Denominator => {
                            if denominator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denominator"));
                            }
                            denominator__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Ratio {
                    numerator: numerator__.unwrap_or_default(),
                    denominator: denominator__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.Ratio", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_source.is_some() {
            len += 1;
        }
        if self.spend_height != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.SpendInfo", len)?;
        if let Some(v) = self.note_source.as_ref() {
            struct_ser.serialize_field("noteSource", v)?;
        }
        if self.spend_height != 0 {
            struct_ser.serialize_field("spendHeight", ToString::to_string(&self.spend_height).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_source",
            "noteSource",
            "spend_height",
            "spendHeight",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NoteSource,
            SpendHeight,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "noteSource" | "note_source" => Ok(GeneratedField::NoteSource),
                            "spendHeight" | "spend_height" => Ok(GeneratedField::SpendHeight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.SpendInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_source__ = None;
                let mut spend_height__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NoteSource => {
                            if note_source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noteSource"));
                            }
                            note_source__ = map.next_value()?;
                        }
                        GeneratedField::SpendHeight => {
                            if spend_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendHeight"));
                            }
                            spend_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SpendInfo {
                    note_source: note_source__,
                    spend_height: spend_height__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.SpendInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StatePayload {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.state_payload.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.StatePayload", len)?;
        if let Some(v) = self.state_payload.as_ref() {
            match v {
                state_payload::StatePayload::RolledUp(v) => {
                    struct_ser.serialize_field("rolledUp", v)?;
                }
                state_payload::StatePayload::Note(v) => {
                    struct_ser.serialize_field("note", v)?;
                }
                state_payload::StatePayload::Swap(v) => {
                    struct_ser.serialize_field("swap", v)?;
                }
                state_payload::StatePayload::Position(v) => {
                    struct_ser.serialize_field("position", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StatePayload {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rolled_up",
            "rolledUp",
            "note",
            "swap",
            "position",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RolledUp,
            Note,
            Swap,
            Position,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rolledUp" | "rolled_up" => Ok(GeneratedField::RolledUp),
                            "note" => Ok(GeneratedField::Note),
                            "swap" => Ok(GeneratedField::Swap),
                            "position" => Ok(GeneratedField::Position),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StatePayload;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.StatePayload")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<StatePayload, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut state_payload__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RolledUp => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rolledUp"));
                            }
                            state_payload__ = map.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::RolledUp)
;
                        }
                        GeneratedField::Note => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            state_payload__ = map.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::Note)
;
                        }
                        GeneratedField::Swap => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            state_payload__ = map.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::Swap)
;
                        }
                        GeneratedField::Position => {
                            if state_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            state_payload__ = map.next_value::<::std::option::Option<_>>()?.map(state_payload::StatePayload::Position)
;
                        }
                    }
                }
                Ok(StatePayload {
                    state_payload: state_payload__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.StatePayload", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::Note {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.source.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.StatePayload.Note", len)?;
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::Note {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Source,
            Note,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "source" => Ok(GeneratedField::Source),
                            "note" => Ok(GeneratedField::Note),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = state_payload::Note;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.StatePayload.Note")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<state_payload::Note, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source__ = None;
                let mut note__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                    }
                }
                Ok(state_payload::Note {
                    source: source__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.StatePayload.Note", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::Position {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.lp_nft.is_some() {
            len += 1;
        }
        if self.commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.StatePayload.Position", len)?;
        if let Some(v) = self.lp_nft.as_ref() {
            struct_ser.serialize_field("lpNft", v)?;
        }
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::Position {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "lp_nft",
            "lpNft",
            "commitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            LpNft,
            Commitment,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "lpNft" | "lp_nft" => Ok(GeneratedField::LpNft),
                            "commitment" => Ok(GeneratedField::Commitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = state_payload::Position;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.StatePayload.Position")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<state_payload::Position, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut lp_nft__ = None;
                let mut commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::LpNft => {
                            if lp_nft__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lpNft"));
                            }
                            lp_nft__ = map.next_value()?;
                        }
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map.next_value()?;
                        }
                    }
                }
                Ok(state_payload::Position {
                    lp_nft: lp_nft__,
                    commitment: commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.StatePayload.Position", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::RolledUp {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.StatePayload.RolledUp", len)?;
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::RolledUp {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitment,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "commitment" => Ok(GeneratedField::Commitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = state_payload::RolledUp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.StatePayload.RolledUp")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<state_payload::RolledUp, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map.next_value()?;
                        }
                    }
                }
                Ok(state_payload::RolledUp {
                    commitment: commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.StatePayload.RolledUp", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for state_payload::Swap {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.source.is_some() {
            len += 1;
        }
        if self.swap.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.chain.v1alpha1.StatePayload.Swap", len)?;
        if let Some(v) = self.source.as_ref() {
            struct_ser.serialize_field("source", v)?;
        }
        if let Some(v) = self.swap.as_ref() {
            struct_ser.serialize_field("swap", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for state_payload::Swap {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "source",
            "swap",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Source,
            Swap,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "source" => Ok(GeneratedField::Source),
                            "swap" => Ok(GeneratedField::Swap),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = state_payload::Swap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.chain.v1alpha1.StatePayload.Swap")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<state_payload::Swap, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut source__ = None;
                let mut swap__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map.next_value()?;
                        }
                        GeneratedField::Swap => {
                            if swap__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            swap__ = map.next_value()?;
                        }
                    }
                }
                Ok(state_payload::Swap {
                    source: source__,
                    swap: swap__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.chain.v1alpha1.StatePayload.Swap", FIELDS, GeneratedVisitor)
    }
}
