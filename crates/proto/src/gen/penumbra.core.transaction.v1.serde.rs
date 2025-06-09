impl serde::Serialize for Action {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.action.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.Action", len)?;
        if let Some(v) = self.action.as_ref() {
            match v {
                action::Action::Spend(v) => {
                    struct_ser.serialize_field("spend", v)?;
                }
                action::Action::Output(v) => {
                    struct_ser.serialize_field("output", v)?;
                }
                action::Action::Swap(v) => {
                    struct_ser.serialize_field("swap", v)?;
                }
                action::Action::SwapClaim(v) => {
                    struct_ser.serialize_field("swapClaim", v)?;
                }
                action::Action::ValidatorDefinition(v) => {
                    struct_ser.serialize_field("validatorDefinition", v)?;
                }
                action::Action::IbcRelayAction(v) => {
                    struct_ser.serialize_field("ibcRelayAction", v)?;
                }
                action::Action::ProposalSubmit(v) => {
                    struct_ser.serialize_field("proposalSubmit", v)?;
                }
                action::Action::ProposalWithdraw(v) => {
                    struct_ser.serialize_field("proposalWithdraw", v)?;
                }
                action::Action::ValidatorVote(v) => {
                    struct_ser.serialize_field("validatorVote", v)?;
                }
                action::Action::DelegatorVote(v) => {
                    struct_ser.serialize_field("delegatorVote", v)?;
                }
                action::Action::ProposalDepositClaim(v) => {
                    struct_ser.serialize_field("proposalDepositClaim", v)?;
                }
                action::Action::PositionOpen(v) => {
                    struct_ser.serialize_field("positionOpen", v)?;
                }
                action::Action::PositionClose(v) => {
                    struct_ser.serialize_field("positionClose", v)?;
                }
                action::Action::PositionWithdraw(v) => {
                    struct_ser.serialize_field("positionWithdraw", v)?;
                }
                action::Action::PositionRewardClaim(v) => {
                    struct_ser.serialize_field("positionRewardClaim", v)?;
                }
                action::Action::Delegate(v) => {
                    struct_ser.serialize_field("delegate", v)?;
                }
                action::Action::Undelegate(v) => {
                    struct_ser.serialize_field("undelegate", v)?;
                }
                action::Action::UndelegateClaim(v) => {
                    struct_ser.serialize_field("undelegateClaim", v)?;
                }
                action::Action::CommunityPoolSpend(v) => {
                    struct_ser.serialize_field("communityPoolSpend", v)?;
                }
                action::Action::CommunityPoolOutput(v) => {
                    struct_ser.serialize_field("communityPoolOutput", v)?;
                }
                action::Action::CommunityPoolDeposit(v) => {
                    struct_ser.serialize_field("communityPoolDeposit", v)?;
                }
                action::Action::ActionDutchAuctionSchedule(v) => {
                    struct_ser.serialize_field("actionDutchAuctionSchedule", v)?;
                }
                action::Action::ActionDutchAuctionEnd(v) => {
                    struct_ser.serialize_field("actionDutchAuctionEnd", v)?;
                }
                action::Action::ActionDutchAuctionWithdraw(v) => {
                    struct_ser.serialize_field("actionDutchAuctionWithdraw", v)?;
                }
                action::Action::ActionLiquidityTournamentVote(v) => {
                    struct_ser.serialize_field("actionLiquidityTournamentVote", v)?;
                }
                action::Action::Ics20Withdrawal(v) => {
                    struct_ser.serialize_field("ics20Withdrawal", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Action {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "output",
            "swap",
            "swap_claim",
            "swapClaim",
            "validator_definition",
            "validatorDefinition",
            "ibc_relay_action",
            "ibcRelayAction",
            "proposal_submit",
            "proposalSubmit",
            "proposal_withdraw",
            "proposalWithdraw",
            "validator_vote",
            "validatorVote",
            "delegator_vote",
            "delegatorVote",
            "proposal_deposit_claim",
            "proposalDepositClaim",
            "position_open",
            "positionOpen",
            "position_close",
            "positionClose",
            "position_withdraw",
            "positionWithdraw",
            "position_reward_claim",
            "positionRewardClaim",
            "delegate",
            "undelegate",
            "undelegate_claim",
            "undelegateClaim",
            "community_pool_spend",
            "communityPoolSpend",
            "community_pool_output",
            "communityPoolOutput",
            "community_pool_deposit",
            "communityPoolDeposit",
            "action_dutch_auction_schedule",
            "actionDutchAuctionSchedule",
            "action_dutch_auction_end",
            "actionDutchAuctionEnd",
            "action_dutch_auction_withdraw",
            "actionDutchAuctionWithdraw",
            "action_liquidity_tournament_vote",
            "actionLiquidityTournamentVote",
            "ics20_withdrawal",
            "ics20Withdrawal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            Swap,
            SwapClaim,
            ValidatorDefinition,
            IbcRelayAction,
            ProposalSubmit,
            ProposalWithdraw,
            ValidatorVote,
            DelegatorVote,
            ProposalDepositClaim,
            PositionOpen,
            PositionClose,
            PositionWithdraw,
            PositionRewardClaim,
            Delegate,
            Undelegate,
            UndelegateClaim,
            CommunityPoolSpend,
            CommunityPoolOutput,
            CommunityPoolDeposit,
            ActionDutchAuctionSchedule,
            ActionDutchAuctionEnd,
            ActionDutchAuctionWithdraw,
            ActionLiquidityTournamentVote,
            Ics20Withdrawal,
            __SkipField__,
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
                            "spend" => Ok(GeneratedField::Spend),
                            "output" => Ok(GeneratedField::Output),
                            "swap" => Ok(GeneratedField::Swap),
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "validatorDefinition" | "validator_definition" => Ok(GeneratedField::ValidatorDefinition),
                            "ibcRelayAction" | "ibc_relay_action" => Ok(GeneratedField::IbcRelayAction),
                            "proposalSubmit" | "proposal_submit" => Ok(GeneratedField::ProposalSubmit),
                            "proposalWithdraw" | "proposal_withdraw" => Ok(GeneratedField::ProposalWithdraw),
                            "validatorVote" | "validator_vote" => Ok(GeneratedField::ValidatorVote),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "proposalDepositClaim" | "proposal_deposit_claim" => Ok(GeneratedField::ProposalDepositClaim),
                            "positionOpen" | "position_open" => Ok(GeneratedField::PositionOpen),
                            "positionClose" | "position_close" => Ok(GeneratedField::PositionClose),
                            "positionWithdraw" | "position_withdraw" => Ok(GeneratedField::PositionWithdraw),
                            "positionRewardClaim" | "position_reward_claim" => Ok(GeneratedField::PositionRewardClaim),
                            "delegate" => Ok(GeneratedField::Delegate),
                            "undelegate" => Ok(GeneratedField::Undelegate),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "communityPoolSpend" | "community_pool_spend" => Ok(GeneratedField::CommunityPoolSpend),
                            "communityPoolOutput" | "community_pool_output" => Ok(GeneratedField::CommunityPoolOutput),
                            "communityPoolDeposit" | "community_pool_deposit" => Ok(GeneratedField::CommunityPoolDeposit),
                            "actionDutchAuctionSchedule" | "action_dutch_auction_schedule" => Ok(GeneratedField::ActionDutchAuctionSchedule),
                            "actionDutchAuctionEnd" | "action_dutch_auction_end" => Ok(GeneratedField::ActionDutchAuctionEnd),
                            "actionDutchAuctionWithdraw" | "action_dutch_auction_withdraw" => Ok(GeneratedField::ActionDutchAuctionWithdraw),
                            "actionLiquidityTournamentVote" | "action_liquidity_tournament_vote" => Ok(GeneratedField::ActionLiquidityTournamentVote),
                            "ics20Withdrawal" | "ics20_withdrawal" => Ok(GeneratedField::Ics20Withdrawal),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.Action")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Action, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::Spend)
;
                        }
                        GeneratedField::Output => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::Output)
;
                        }
                        GeneratedField::Swap => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::Swap)
;
                        }
                        GeneratedField::SwapClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::SwapClaim)
;
                        }
                        GeneratedField::ValidatorDefinition => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ValidatorDefinition)
;
                        }
                        GeneratedField::IbcRelayAction => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcRelayAction"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::IbcRelayAction)
;
                        }
                        GeneratedField::ProposalSubmit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSubmit"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ProposalSubmit)
;
                        }
                        GeneratedField::ProposalWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalWithdraw"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ProposalWithdraw)
;
                        }
                        GeneratedField::ValidatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ValidatorVote)
;
                        }
                        GeneratedField::DelegatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::DelegatorVote)
;
                        }
                        GeneratedField::ProposalDepositClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ProposalDepositClaim)
;
                        }
                        GeneratedField::PositionOpen => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpen"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionOpen)
;
                        }
                        GeneratedField::PositionClose => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionClose"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionClose)
;
                        }
                        GeneratedField::PositionWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraw"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionWithdraw)
;
                        }
                        GeneratedField::PositionRewardClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionRewardClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionRewardClaim)
;
                        }
                        GeneratedField::Delegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegate"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::Delegate)
;
                        }
                        GeneratedField::Undelegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegate"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::Undelegate)
;
                        }
                        GeneratedField::UndelegateClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::UndelegateClaim)
;
                        }
                        GeneratedField::CommunityPoolSpend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolSpend"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::CommunityPoolSpend)
;
                        }
                        GeneratedField::CommunityPoolOutput => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolOutput"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::CommunityPoolOutput)
;
                        }
                        GeneratedField::CommunityPoolDeposit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolDeposit"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::CommunityPoolDeposit)
;
                        }
                        GeneratedField::ActionDutchAuctionSchedule => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionSchedule"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ActionDutchAuctionSchedule)
;
                        }
                        GeneratedField::ActionDutchAuctionEnd => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionEnd"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ActionDutchAuctionEnd)
;
                        }
                        GeneratedField::ActionDutchAuctionWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionWithdraw"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ActionDutchAuctionWithdraw)
;
                        }
                        GeneratedField::ActionLiquidityTournamentVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionLiquidityTournamentVote"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::ActionLiquidityTournamentVote)
;
                        }
                        GeneratedField::Ics20Withdrawal => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Withdrawal"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action::Action::Ics20Withdrawal)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Action {
                    action: action__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.Action", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.action.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.ActionPlan", len)?;
        if let Some(v) = self.action.as_ref() {
            match v {
                action_plan::Action::Spend(v) => {
                    struct_ser.serialize_field("spend", v)?;
                }
                action_plan::Action::Output(v) => {
                    struct_ser.serialize_field("output", v)?;
                }
                action_plan::Action::Swap(v) => {
                    struct_ser.serialize_field("swap", v)?;
                }
                action_plan::Action::SwapClaim(v) => {
                    struct_ser.serialize_field("swapClaim", v)?;
                }
                action_plan::Action::ValidatorDefinition(v) => {
                    struct_ser.serialize_field("validatorDefinition", v)?;
                }
                action_plan::Action::IbcRelayAction(v) => {
                    struct_ser.serialize_field("ibcRelayAction", v)?;
                }
                action_plan::Action::ProposalSubmit(v) => {
                    struct_ser.serialize_field("proposalSubmit", v)?;
                }
                action_plan::Action::ProposalWithdraw(v) => {
                    struct_ser.serialize_field("proposalWithdraw", v)?;
                }
                action_plan::Action::ValidatorVote(v) => {
                    struct_ser.serialize_field("validatorVote", v)?;
                }
                action_plan::Action::DelegatorVote(v) => {
                    struct_ser.serialize_field("delegatorVote", v)?;
                }
                action_plan::Action::ProposalDepositClaim(v) => {
                    struct_ser.serialize_field("proposalDepositClaim", v)?;
                }
                action_plan::Action::Ics20Withdrawal(v) => {
                    struct_ser.serialize_field("ics20Withdrawal", v)?;
                }
                action_plan::Action::PositionOpen(v) => {
                    struct_ser.serialize_field("positionOpen", v)?;
                }
                action_plan::Action::PositionOpenPlan(v) => {
                    struct_ser.serialize_field("positionOpenPlan", v)?;
                }
                action_plan::Action::PositionClose(v) => {
                    struct_ser.serialize_field("positionClose", v)?;
                }
                action_plan::Action::PositionWithdraw(v) => {
                    struct_ser.serialize_field("positionWithdraw", v)?;
                }
                action_plan::Action::PositionRewardClaim(v) => {
                    struct_ser.serialize_field("positionRewardClaim", v)?;
                }
                action_plan::Action::Delegate(v) => {
                    struct_ser.serialize_field("delegate", v)?;
                }
                action_plan::Action::Undelegate(v) => {
                    struct_ser.serialize_field("undelegate", v)?;
                }
                action_plan::Action::UndelegateClaim(v) => {
                    struct_ser.serialize_field("undelegateClaim", v)?;
                }
                action_plan::Action::CommunityPoolSpend(v) => {
                    struct_ser.serialize_field("communityPoolSpend", v)?;
                }
                action_plan::Action::CommunityPoolOutput(v) => {
                    struct_ser.serialize_field("communityPoolOutput", v)?;
                }
                action_plan::Action::CommunityPoolDeposit(v) => {
                    struct_ser.serialize_field("communityPoolDeposit", v)?;
                }
                action_plan::Action::ActionDutchAuctionSchedule(v) => {
                    struct_ser.serialize_field("actionDutchAuctionSchedule", v)?;
                }
                action_plan::Action::ActionDutchAuctionEnd(v) => {
                    struct_ser.serialize_field("actionDutchAuctionEnd", v)?;
                }
                action_plan::Action::ActionDutchAuctionWithdraw(v) => {
                    struct_ser.serialize_field("actionDutchAuctionWithdraw", v)?;
                }
                action_plan::Action::ActionLiquidityTournamentVote(v) => {
                    struct_ser.serialize_field("actionLiquidityTournamentVote", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "output",
            "swap",
            "swap_claim",
            "swapClaim",
            "validator_definition",
            "validatorDefinition",
            "ibc_relay_action",
            "ibcRelayAction",
            "proposal_submit",
            "proposalSubmit",
            "proposal_withdraw",
            "proposalWithdraw",
            "validator_vote",
            "validatorVote",
            "delegator_vote",
            "delegatorVote",
            "proposal_deposit_claim",
            "proposalDepositClaim",
            "ics20_withdrawal",
            "ics20Withdrawal",
            "position_open",
            "positionOpen",
            "position_open_plan",
            "positionOpenPlan",
            "position_close",
            "positionClose",
            "position_withdraw",
            "positionWithdraw",
            "position_reward_claim",
            "positionRewardClaim",
            "delegate",
            "undelegate",
            "undelegate_claim",
            "undelegateClaim",
            "community_pool_spend",
            "communityPoolSpend",
            "community_pool_output",
            "communityPoolOutput",
            "community_pool_deposit",
            "communityPoolDeposit",
            "action_dutch_auction_schedule",
            "actionDutchAuctionSchedule",
            "action_dutch_auction_end",
            "actionDutchAuctionEnd",
            "action_dutch_auction_withdraw",
            "actionDutchAuctionWithdraw",
            "action_liquidity_tournament_vote",
            "actionLiquidityTournamentVote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            Swap,
            SwapClaim,
            ValidatorDefinition,
            IbcRelayAction,
            ProposalSubmit,
            ProposalWithdraw,
            ValidatorVote,
            DelegatorVote,
            ProposalDepositClaim,
            Ics20Withdrawal,
            PositionOpen,
            PositionOpenPlan,
            PositionClose,
            PositionWithdraw,
            PositionRewardClaim,
            Delegate,
            Undelegate,
            UndelegateClaim,
            CommunityPoolSpend,
            CommunityPoolOutput,
            CommunityPoolDeposit,
            ActionDutchAuctionSchedule,
            ActionDutchAuctionEnd,
            ActionDutchAuctionWithdraw,
            ActionLiquidityTournamentVote,
            __SkipField__,
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
                            "spend" => Ok(GeneratedField::Spend),
                            "output" => Ok(GeneratedField::Output),
                            "swap" => Ok(GeneratedField::Swap),
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "validatorDefinition" | "validator_definition" => Ok(GeneratedField::ValidatorDefinition),
                            "ibcRelayAction" | "ibc_relay_action" => Ok(GeneratedField::IbcRelayAction),
                            "proposalSubmit" | "proposal_submit" => Ok(GeneratedField::ProposalSubmit),
                            "proposalWithdraw" | "proposal_withdraw" => Ok(GeneratedField::ProposalWithdraw),
                            "validatorVote" | "validator_vote" => Ok(GeneratedField::ValidatorVote),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "proposalDepositClaim" | "proposal_deposit_claim" => Ok(GeneratedField::ProposalDepositClaim),
                            "ics20Withdrawal" | "ics20_withdrawal" => Ok(GeneratedField::Ics20Withdrawal),
                            "positionOpen" | "position_open" => Ok(GeneratedField::PositionOpen),
                            "positionOpenPlan" | "position_open_plan" => Ok(GeneratedField::PositionOpenPlan),
                            "positionClose" | "position_close" => Ok(GeneratedField::PositionClose),
                            "positionWithdraw" | "position_withdraw" => Ok(GeneratedField::PositionWithdraw),
                            "positionRewardClaim" | "position_reward_claim" => Ok(GeneratedField::PositionRewardClaim),
                            "delegate" => Ok(GeneratedField::Delegate),
                            "undelegate" => Ok(GeneratedField::Undelegate),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "communityPoolSpend" | "community_pool_spend" => Ok(GeneratedField::CommunityPoolSpend),
                            "communityPoolOutput" | "community_pool_output" => Ok(GeneratedField::CommunityPoolOutput),
                            "communityPoolDeposit" | "community_pool_deposit" => Ok(GeneratedField::CommunityPoolDeposit),
                            "actionDutchAuctionSchedule" | "action_dutch_auction_schedule" => Ok(GeneratedField::ActionDutchAuctionSchedule),
                            "actionDutchAuctionEnd" | "action_dutch_auction_end" => Ok(GeneratedField::ActionDutchAuctionEnd),
                            "actionDutchAuctionWithdraw" | "action_dutch_auction_withdraw" => Ok(GeneratedField::ActionDutchAuctionWithdraw),
                            "actionLiquidityTournamentVote" | "action_liquidity_tournament_vote" => Ok(GeneratedField::ActionLiquidityTournamentVote),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.ActionPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Spend)
;
                        }
                        GeneratedField::Output => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Output)
;
                        }
                        GeneratedField::Swap => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Swap)
;
                        }
                        GeneratedField::SwapClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::SwapClaim)
;
                        }
                        GeneratedField::ValidatorDefinition => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ValidatorDefinition)
;
                        }
                        GeneratedField::IbcRelayAction => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcRelayAction"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::IbcRelayAction)
;
                        }
                        GeneratedField::ProposalSubmit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSubmit"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ProposalSubmit)
;
                        }
                        GeneratedField::ProposalWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalWithdraw"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ProposalWithdraw)
;
                        }
                        GeneratedField::ValidatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ValidatorVote)
;
                        }
                        GeneratedField::DelegatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::DelegatorVote)
;
                        }
                        GeneratedField::ProposalDepositClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ProposalDepositClaim)
;
                        }
                        GeneratedField::Ics20Withdrawal => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Withdrawal"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Ics20Withdrawal)
;
                        }
                        GeneratedField::PositionOpen => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpen"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionOpen)
;
                        }
                        GeneratedField::PositionOpenPlan => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpenPlan"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionOpenPlan)
;
                        }
                        GeneratedField::PositionClose => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionClose"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionClose)
;
                        }
                        GeneratedField::PositionWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraw"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionWithdraw)
;
                        }
                        GeneratedField::PositionRewardClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionRewardClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionRewardClaim)
;
                        }
                        GeneratedField::Delegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegate"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Delegate)
;
                        }
                        GeneratedField::Undelegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegate"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Undelegate)
;
                        }
                        GeneratedField::UndelegateClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::UndelegateClaim)
;
                        }
                        GeneratedField::CommunityPoolSpend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolSpend"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::CommunityPoolSpend)
;
                        }
                        GeneratedField::CommunityPoolOutput => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolOutput"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::CommunityPoolOutput)
;
                        }
                        GeneratedField::CommunityPoolDeposit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolDeposit"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::CommunityPoolDeposit)
;
                        }
                        GeneratedField::ActionDutchAuctionSchedule => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionSchedule"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ActionDutchAuctionSchedule)
;
                        }
                        GeneratedField::ActionDutchAuctionEnd => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionEnd"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ActionDutchAuctionEnd)
;
                        }
                        GeneratedField::ActionDutchAuctionWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionWithdraw"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ActionDutchAuctionWithdraw)
;
                        }
                        GeneratedField::ActionLiquidityTournamentVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionLiquidityTournamentVote"));
                            }
                            action__ = map_.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ActionLiquidityTournamentVote)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionPlan {
                    action: action__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.ActionPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.action_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.ActionView", len)?;
        if let Some(v) = self.action_view.as_ref() {
            match v {
                action_view::ActionView::Spend(v) => {
                    struct_ser.serialize_field("spend", v)?;
                }
                action_view::ActionView::Output(v) => {
                    struct_ser.serialize_field("output", v)?;
                }
                action_view::ActionView::Swap(v) => {
                    struct_ser.serialize_field("swap", v)?;
                }
                action_view::ActionView::SwapClaim(v) => {
                    struct_ser.serialize_field("swapClaim", v)?;
                }
                action_view::ActionView::DelegatorVote(v) => {
                    struct_ser.serialize_field("delegatorVote", v)?;
                }
                action_view::ActionView::PositionOpenView(v) => {
                    struct_ser.serialize_field("positionOpenView", v)?;
                }
                action_view::ActionView::ValidatorDefinition(v) => {
                    struct_ser.serialize_field("validatorDefinition", v)?;
                }
                action_view::ActionView::IbcRelayAction(v) => {
                    struct_ser.serialize_field("ibcRelayAction", v)?;
                }
                action_view::ActionView::ProposalSubmit(v) => {
                    struct_ser.serialize_field("proposalSubmit", v)?;
                }
                action_view::ActionView::ProposalWithdraw(v) => {
                    struct_ser.serialize_field("proposalWithdraw", v)?;
                }
                action_view::ActionView::ValidatorVote(v) => {
                    struct_ser.serialize_field("validatorVote", v)?;
                }
                action_view::ActionView::ProposalDepositClaim(v) => {
                    struct_ser.serialize_field("proposalDepositClaim", v)?;
                }
                action_view::ActionView::PositionOpen(v) => {
                    struct_ser.serialize_field("positionOpen", v)?;
                }
                action_view::ActionView::PositionClose(v) => {
                    struct_ser.serialize_field("positionClose", v)?;
                }
                action_view::ActionView::PositionWithdraw(v) => {
                    struct_ser.serialize_field("positionWithdraw", v)?;
                }
                action_view::ActionView::PositionRewardClaim(v) => {
                    struct_ser.serialize_field("positionRewardClaim", v)?;
                }
                action_view::ActionView::Delegate(v) => {
                    struct_ser.serialize_field("delegate", v)?;
                }
                action_view::ActionView::Undelegate(v) => {
                    struct_ser.serialize_field("undelegate", v)?;
                }
                action_view::ActionView::CommunityPoolSpend(v) => {
                    struct_ser.serialize_field("communityPoolSpend", v)?;
                }
                action_view::ActionView::CommunityPoolOutput(v) => {
                    struct_ser.serialize_field("communityPoolOutput", v)?;
                }
                action_view::ActionView::CommunityPoolDeposit(v) => {
                    struct_ser.serialize_field("communityPoolDeposit", v)?;
                }
                action_view::ActionView::ActionDutchAuctionSchedule(v) => {
                    struct_ser.serialize_field("actionDutchAuctionSchedule", v)?;
                }
                action_view::ActionView::ActionDutchAuctionEnd(v) => {
                    struct_ser.serialize_field("actionDutchAuctionEnd", v)?;
                }
                action_view::ActionView::ActionDutchAuctionWithdraw(v) => {
                    struct_ser.serialize_field("actionDutchAuctionWithdraw", v)?;
                }
                action_view::ActionView::UndelegateClaim(v) => {
                    struct_ser.serialize_field("undelegateClaim", v)?;
                }
                action_view::ActionView::ActionLiquidityTournamentVote(v) => {
                    struct_ser.serialize_field("actionLiquidityTournamentVote", v)?;
                }
                action_view::ActionView::Ics20Withdrawal(v) => {
                    struct_ser.serialize_field("ics20Withdrawal", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "output",
            "swap",
            "swap_claim",
            "swapClaim",
            "delegator_vote",
            "delegatorVote",
            "position_open_view",
            "positionOpenView",
            "validator_definition",
            "validatorDefinition",
            "ibc_relay_action",
            "ibcRelayAction",
            "proposal_submit",
            "proposalSubmit",
            "proposal_withdraw",
            "proposalWithdraw",
            "validator_vote",
            "validatorVote",
            "proposal_deposit_claim",
            "proposalDepositClaim",
            "position_open",
            "positionOpen",
            "position_close",
            "positionClose",
            "position_withdraw",
            "positionWithdraw",
            "position_reward_claim",
            "positionRewardClaim",
            "delegate",
            "undelegate",
            "community_pool_spend",
            "communityPoolSpend",
            "community_pool_output",
            "communityPoolOutput",
            "community_pool_deposit",
            "communityPoolDeposit",
            "action_dutch_auction_schedule",
            "actionDutchAuctionSchedule",
            "action_dutch_auction_end",
            "actionDutchAuctionEnd",
            "action_dutch_auction_withdraw",
            "actionDutchAuctionWithdraw",
            "undelegate_claim",
            "undelegateClaim",
            "action_liquidity_tournament_vote",
            "actionLiquidityTournamentVote",
            "ics20_withdrawal",
            "ics20Withdrawal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            Swap,
            SwapClaim,
            DelegatorVote,
            PositionOpenView,
            ValidatorDefinition,
            IbcRelayAction,
            ProposalSubmit,
            ProposalWithdraw,
            ValidatorVote,
            ProposalDepositClaim,
            PositionOpen,
            PositionClose,
            PositionWithdraw,
            PositionRewardClaim,
            Delegate,
            Undelegate,
            CommunityPoolSpend,
            CommunityPoolOutput,
            CommunityPoolDeposit,
            ActionDutchAuctionSchedule,
            ActionDutchAuctionEnd,
            ActionDutchAuctionWithdraw,
            UndelegateClaim,
            ActionLiquidityTournamentVote,
            Ics20Withdrawal,
            __SkipField__,
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
                            "spend" => Ok(GeneratedField::Spend),
                            "output" => Ok(GeneratedField::Output),
                            "swap" => Ok(GeneratedField::Swap),
                            "swapClaim" | "swap_claim" => Ok(GeneratedField::SwapClaim),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "positionOpenView" | "position_open_view" => Ok(GeneratedField::PositionOpenView),
                            "validatorDefinition" | "validator_definition" => Ok(GeneratedField::ValidatorDefinition),
                            "ibcRelayAction" | "ibc_relay_action" => Ok(GeneratedField::IbcRelayAction),
                            "proposalSubmit" | "proposal_submit" => Ok(GeneratedField::ProposalSubmit),
                            "proposalWithdraw" | "proposal_withdraw" => Ok(GeneratedField::ProposalWithdraw),
                            "validatorVote" | "validator_vote" => Ok(GeneratedField::ValidatorVote),
                            "proposalDepositClaim" | "proposal_deposit_claim" => Ok(GeneratedField::ProposalDepositClaim),
                            "positionOpen" | "position_open" => Ok(GeneratedField::PositionOpen),
                            "positionClose" | "position_close" => Ok(GeneratedField::PositionClose),
                            "positionWithdraw" | "position_withdraw" => Ok(GeneratedField::PositionWithdraw),
                            "positionRewardClaim" | "position_reward_claim" => Ok(GeneratedField::PositionRewardClaim),
                            "delegate" => Ok(GeneratedField::Delegate),
                            "undelegate" => Ok(GeneratedField::Undelegate),
                            "communityPoolSpend" | "community_pool_spend" => Ok(GeneratedField::CommunityPoolSpend),
                            "communityPoolOutput" | "community_pool_output" => Ok(GeneratedField::CommunityPoolOutput),
                            "communityPoolDeposit" | "community_pool_deposit" => Ok(GeneratedField::CommunityPoolDeposit),
                            "actionDutchAuctionSchedule" | "action_dutch_auction_schedule" => Ok(GeneratedField::ActionDutchAuctionSchedule),
                            "actionDutchAuctionEnd" | "action_dutch_auction_end" => Ok(GeneratedField::ActionDutchAuctionEnd),
                            "actionDutchAuctionWithdraw" | "action_dutch_auction_withdraw" => Ok(GeneratedField::ActionDutchAuctionWithdraw),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "actionLiquidityTournamentVote" | "action_liquidity_tournament_vote" => Ok(GeneratedField::ActionLiquidityTournamentVote),
                            "ics20Withdrawal" | "ics20_withdrawal" => Ok(GeneratedField::Ics20Withdrawal),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.ActionView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Spend)
;
                        }
                        GeneratedField::Output => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Output)
;
                        }
                        GeneratedField::Swap => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Swap)
;
                        }
                        GeneratedField::SwapClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::SwapClaim)
;
                        }
                        GeneratedField::DelegatorVote => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::DelegatorVote)
;
                        }
                        GeneratedField::PositionOpenView => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpenView"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionOpenView)
;
                        }
                        GeneratedField::ValidatorDefinition => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ValidatorDefinition)
;
                        }
                        GeneratedField::IbcRelayAction => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcRelayAction"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::IbcRelayAction)
;
                        }
                        GeneratedField::ProposalSubmit => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSubmit"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ProposalSubmit)
;
                        }
                        GeneratedField::ProposalWithdraw => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalWithdraw"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ProposalWithdraw)
;
                        }
                        GeneratedField::ValidatorVote => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ValidatorVote)
;
                        }
                        GeneratedField::ProposalDepositClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositClaim"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ProposalDepositClaim)
;
                        }
                        GeneratedField::PositionOpen => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpen"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionOpen)
;
                        }
                        GeneratedField::PositionClose => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionClose"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionClose)
;
                        }
                        GeneratedField::PositionWithdraw => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraw"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionWithdraw)
;
                        }
                        GeneratedField::PositionRewardClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionRewardClaim"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionRewardClaim)
;
                        }
                        GeneratedField::Delegate => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegate"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Delegate)
;
                        }
                        GeneratedField::Undelegate => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegate"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Undelegate)
;
                        }
                        GeneratedField::CommunityPoolSpend => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolSpend"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::CommunityPoolSpend)
;
                        }
                        GeneratedField::CommunityPoolOutput => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolOutput"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::CommunityPoolOutput)
;
                        }
                        GeneratedField::CommunityPoolDeposit => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("communityPoolDeposit"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::CommunityPoolDeposit)
;
                        }
                        GeneratedField::ActionDutchAuctionSchedule => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionSchedule"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ActionDutchAuctionSchedule)
;
                        }
                        GeneratedField::ActionDutchAuctionEnd => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionEnd"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ActionDutchAuctionEnd)
;
                        }
                        GeneratedField::ActionDutchAuctionWithdraw => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionDutchAuctionWithdraw"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ActionDutchAuctionWithdraw)
;
                        }
                        GeneratedField::UndelegateClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::UndelegateClaim)
;
                        }
                        GeneratedField::ActionLiquidityTournamentVote => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionLiquidityTournamentVote"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ActionLiquidityTournamentVote)
;
                        }
                        GeneratedField::Ics20Withdrawal => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Withdrawal"));
                            }
                            action_view__ = map_.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Ics20Withdrawal)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionView {
                    action_view: action_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.ActionView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthorizationData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.effect_hash.is_some() {
            len += 1;
        }
        if !self.spend_auths.is_empty() {
            len += 1;
        }
        if !self.delegator_vote_auths.is_empty() {
            len += 1;
        }
        if !self.lqt_vote_auths.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.AuthorizationData", len)?;
        if let Some(v) = self.effect_hash.as_ref() {
            struct_ser.serialize_field("effectHash", v)?;
        }
        if !self.spend_auths.is_empty() {
            struct_ser.serialize_field("spendAuths", &self.spend_auths)?;
        }
        if !self.delegator_vote_auths.is_empty() {
            struct_ser.serialize_field("delegatorVoteAuths", &self.delegator_vote_auths)?;
        }
        if !self.lqt_vote_auths.is_empty() {
            struct_ser.serialize_field("lqtVoteAuths", &self.lqt_vote_auths)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthorizationData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "effect_hash",
            "effectHash",
            "spend_auths",
            "spendAuths",
            "delegator_vote_auths",
            "delegatorVoteAuths",
            "lqt_vote_auths",
            "lqtVoteAuths",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EffectHash,
            SpendAuths,
            DelegatorVoteAuths,
            LqtVoteAuths,
            __SkipField__,
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
                            "effectHash" | "effect_hash" => Ok(GeneratedField::EffectHash),
                            "spendAuths" | "spend_auths" => Ok(GeneratedField::SpendAuths),
                            "delegatorVoteAuths" | "delegator_vote_auths" => Ok(GeneratedField::DelegatorVoteAuths),
                            "lqtVoteAuths" | "lqt_vote_auths" => Ok(GeneratedField::LqtVoteAuths),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthorizationData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.AuthorizationData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthorizationData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut effect_hash__ = None;
                let mut spend_auths__ = None;
                let mut delegator_vote_auths__ = None;
                let mut lqt_vote_auths__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EffectHash => {
                            if effect_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effectHash"));
                            }
                            effect_hash__ = map_.next_value()?;
                        }
                        GeneratedField::SpendAuths => {
                            if spend_auths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendAuths"));
                            }
                            spend_auths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DelegatorVoteAuths => {
                            if delegator_vote_auths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVoteAuths"));
                            }
                            delegator_vote_auths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::LqtVoteAuths => {
                            if lqt_vote_auths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lqtVoteAuths"));
                            }
                            lqt_vote_auths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AuthorizationData {
                    effect_hash: effect_hash__,
                    spend_auths: spend_auths__.unwrap_or_default(),
                    delegator_vote_auths: delegator_vote_auths__.unwrap_or_default(),
                    lqt_vote_auths: lqt_vote_auths__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.AuthorizationData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CluePlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        if !self.rseed.is_empty() {
            len += 1;
        }
        if self.precision_bits != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.CluePlan", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if !self.rseed.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        if self.precision_bits != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("precisionBits", ToString::to_string(&self.precision_bits).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CluePlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "rseed",
            "precision_bits",
            "precisionBits",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Rseed,
            PrecisionBits,
            __SkipField__,
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
                            "address" => Ok(GeneratedField::Address),
                            "rseed" => Ok(GeneratedField::Rseed),
                            "precisionBits" | "precision_bits" => Ok(GeneratedField::PrecisionBits),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CluePlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.CluePlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CluePlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut rseed__ = None;
                let mut precision_bits__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PrecisionBits => {
                            if precision_bits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("precisionBits"));
                            }
                            precision_bits__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CluePlan {
                    address: address__,
                    rseed: rseed__.unwrap_or_default(),
                    precision_bits: precision_bits__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.CluePlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DetectionData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fmd_clues.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.DetectionData", len)?;
        if !self.fmd_clues.is_empty() {
            struct_ser.serialize_field("fmdClues", &self.fmd_clues)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DetectionData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fmd_clues",
            "fmdClues",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FmdClues,
            __SkipField__,
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
                            "fmdClues" | "fmd_clues" => Ok(GeneratedField::FmdClues),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DetectionData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.DetectionData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DetectionData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fmd_clues__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FmdClues => {
                            if fmd_clues__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdClues"));
                            }
                            fmd_clues__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DetectionData {
                    fmd_clues: fmd_clues__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.DetectionData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DetectionDataPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.clue_plans.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.DetectionDataPlan", len)?;
        if !self.clue_plans.is_empty() {
            struct_ser.serialize_field("cluePlans", &self.clue_plans)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DetectionDataPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "clue_plans",
            "cluePlans",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CluePlans,
            __SkipField__,
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
                            "cluePlans" | "clue_plans" => Ok(GeneratedField::CluePlans),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DetectionDataPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.DetectionDataPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DetectionDataPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut clue_plans__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CluePlans => {
                            if clue_plans__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cluePlans"));
                            }
                            clue_plans__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DetectionDataPlan {
                    clue_plans: clue_plans__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.DetectionDataPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MemoCiphertext {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoCiphertext", len)?;
        if !self.inner.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MemoCiphertext {
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
            __SkipField__,
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
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MemoCiphertext;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoCiphertext")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MemoCiphertext, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut inner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Inner => {
                            if inner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inner"));
                            }
                            inner__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MemoCiphertext {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoCiphertext", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MemoPlaintext {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.return_address.is_some() {
            len += 1;
        }
        if !self.text.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoPlaintext", len)?;
        if let Some(v) = self.return_address.as_ref() {
            struct_ser.serialize_field("returnAddress", v)?;
        }
        if !self.text.is_empty() {
            struct_ser.serialize_field("text", &self.text)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MemoPlaintext {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "return_address",
            "returnAddress",
            "text",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ReturnAddress,
            Text,
            __SkipField__,
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
                            "returnAddress" | "return_address" => Ok(GeneratedField::ReturnAddress),
                            "text" => Ok(GeneratedField::Text),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MemoPlaintext;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoPlaintext")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MemoPlaintext, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut return_address__ = None;
                let mut text__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ReturnAddress => {
                            if return_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnAddress"));
                            }
                            return_address__ = map_.next_value()?;
                        }
                        GeneratedField::Text => {
                            if text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            text__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MemoPlaintext {
                    return_address: return_address__,
                    text: text__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoPlaintext", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MemoPlaintextView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.return_address.is_some() {
            len += 1;
        }
        if !self.text.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoPlaintextView", len)?;
        if let Some(v) = self.return_address.as_ref() {
            struct_ser.serialize_field("returnAddress", v)?;
        }
        if !self.text.is_empty() {
            struct_ser.serialize_field("text", &self.text)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MemoPlaintextView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "return_address",
            "returnAddress",
            "text",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ReturnAddress,
            Text,
            __SkipField__,
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
                            "returnAddress" | "return_address" => Ok(GeneratedField::ReturnAddress),
                            "text" => Ok(GeneratedField::Text),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MemoPlaintextView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoPlaintextView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MemoPlaintextView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut return_address__ = None;
                let mut text__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ReturnAddress => {
                            if return_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnAddress"));
                            }
                            return_address__ = map_.next_value()?;
                        }
                        GeneratedField::Text => {
                            if text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            text__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MemoPlaintextView {
                    return_address: return_address__,
                    text: text__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoPlaintextView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MemoPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.plaintext.is_some() {
            len += 1;
        }
        if !self.key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoPlan", len)?;
        if let Some(v) = self.plaintext.as_ref() {
            struct_ser.serialize_field("plaintext", v)?;
        }
        if !self.key.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("key", pbjson::private::base64::encode(&self.key).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MemoPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "plaintext",
            "key",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Plaintext,
            Key,
            __SkipField__,
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
                            "plaintext" => Ok(GeneratedField::Plaintext),
                            "key" => Ok(GeneratedField::Key),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MemoPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MemoPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut plaintext__ = None;
                let mut key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Plaintext => {
                            if plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plaintext"));
                            }
                            plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MemoPlan {
                    plaintext: plaintext__,
                    key: key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MemoView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.memo_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoView", len)?;
        if let Some(v) = self.memo_view.as_ref() {
            match v {
                memo_view::MemoView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                memo_view::MemoView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MemoView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "visible",
            "opaque",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Visible,
            Opaque,
            __SkipField__,
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
                            "visible" => Ok(GeneratedField::Visible),
                            "opaque" => Ok(GeneratedField::Opaque),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MemoView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MemoView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut memo_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if memo_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            memo_view__ = map_.next_value::<::std::option::Option<_>>()?.map(memo_view::MemoView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if memo_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            memo_view__ = map_.next_value::<::std::option::Option<_>>()?.map(memo_view::MemoView::Opaque)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MemoView {
                    memo_view: memo_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for memo_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.ciphertext.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoView.Opaque", len)?;
        if let Some(v) = self.ciphertext.as_ref() {
            struct_ser.serialize_field("ciphertext", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for memo_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ciphertext",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ciphertext,
            __SkipField__,
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
                            "ciphertext" => Ok(GeneratedField::Ciphertext),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = memo_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoView.Opaque")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<memo_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ciphertext__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Ciphertext => {
                            if ciphertext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ciphertext"));
                            }
                            ciphertext__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(memo_view::Opaque {
                    ciphertext: ciphertext__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for memo_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.ciphertext.is_some() {
            len += 1;
        }
        if self.plaintext.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.MemoView.Visible", len)?;
        if let Some(v) = self.ciphertext.as_ref() {
            struct_ser.serialize_field("ciphertext", v)?;
        }
        if let Some(v) = self.plaintext.as_ref() {
            struct_ser.serialize_field("plaintext", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for memo_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ciphertext",
            "plaintext",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ciphertext,
            Plaintext,
            __SkipField__,
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
                            "ciphertext" => Ok(GeneratedField::Ciphertext),
                            "plaintext" => Ok(GeneratedField::Plaintext),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = memo_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.MemoView.Visible")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<memo_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ciphertext__ = None;
                let mut plaintext__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Ciphertext => {
                            if ciphertext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ciphertext"));
                            }
                            ciphertext__ = map_.next_value()?;
                        }
                        GeneratedField::Plaintext => {
                            if plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plaintext"));
                            }
                            plaintext__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(memo_view::Visible {
                    ciphertext: ciphertext__,
                    plaintext: plaintext__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.MemoView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for NullifierWithNote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.NullifierWithNote", len)?;
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for NullifierWithNote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nullifier",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nullifier,
            Note,
            __SkipField__,
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
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "note" => Ok(GeneratedField::Note),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = NullifierWithNote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.NullifierWithNote")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<NullifierWithNote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut note__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(NullifierWithNote {
                    nullifier: nullifier__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.NullifierWithNote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PayloadKeyWithCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.payload_key.is_some() {
            len += 1;
        }
        if self.commitment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.PayloadKeyWithCommitment", len)?;
        if let Some(v) = self.payload_key.as_ref() {
            struct_ser.serialize_field("payloadKey", v)?;
        }
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PayloadKeyWithCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payload_key",
            "payloadKey",
            "commitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayloadKey,
            Commitment,
            __SkipField__,
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
                            "payloadKey" | "payload_key" => Ok(GeneratedField::PayloadKey),
                            "commitment" => Ok(GeneratedField::Commitment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PayloadKeyWithCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.PayloadKeyWithCommitment")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PayloadKeyWithCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload_key__ = None;
                let mut commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PayloadKey => {
                            if payload_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloadKey"));
                            }
                            payload_key__ = map_.next_value()?;
                        }
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PayloadKeyWithCommitment {
                    payload_key: payload_key__,
                    commitment: commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.PayloadKeyWithCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Transaction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.body.is_some() {
            len += 1;
        }
        if self.binding_sig.is_some() {
            len += 1;
        }
        if self.anchor.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.Transaction", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.binding_sig.as_ref() {
            struct_ser.serialize_field("bindingSig", v)?;
        }
        if let Some(v) = self.anchor.as_ref() {
            struct_ser.serialize_field("anchor", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Transaction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "binding_sig",
            "bindingSig",
            "anchor",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            BindingSig,
            Anchor,
            __SkipField__,
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
                            "body" => Ok(GeneratedField::Body),
                            "bindingSig" | "binding_sig" => Ok(GeneratedField::BindingSig),
                            "anchor" => Ok(GeneratedField::Anchor),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.Transaction")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Transaction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut binding_sig__ = None;
                let mut anchor__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map_.next_value()?;
                        }
                        GeneratedField::BindingSig => {
                            if binding_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bindingSig"));
                            }
                            binding_sig__ = map_.next_value()?;
                        }
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Transaction {
                    body: body__,
                    binding_sig: binding_sig__,
                    anchor: anchor__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.Transaction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.actions.is_empty() {
            len += 1;
        }
        if self.transaction_parameters.is_some() {
            len += 1;
        }
        if self.detection_data.is_some() {
            len += 1;
        }
        if self.memo.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionBody", len)?;
        if !self.actions.is_empty() {
            struct_ser.serialize_field("actions", &self.actions)?;
        }
        if let Some(v) = self.transaction_parameters.as_ref() {
            struct_ser.serialize_field("transactionParameters", v)?;
        }
        if let Some(v) = self.detection_data.as_ref() {
            struct_ser.serialize_field("detectionData", v)?;
        }
        if let Some(v) = self.memo.as_ref() {
            struct_ser.serialize_field("memo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "actions",
            "transaction_parameters",
            "transactionParameters",
            "detection_data",
            "detectionData",
            "memo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Actions,
            TransactionParameters,
            DetectionData,
            Memo,
            __SkipField__,
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
                            "actions" => Ok(GeneratedField::Actions),
                            "transactionParameters" | "transaction_parameters" => Ok(GeneratedField::TransactionParameters),
                            "detectionData" | "detection_data" => Ok(GeneratedField::DetectionData),
                            "memo" => Ok(GeneratedField::Memo),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionBody")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut actions__ = None;
                let mut transaction_parameters__ = None;
                let mut detection_data__ = None;
                let mut memo__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Actions => {
                            if actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actions"));
                            }
                            actions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TransactionParameters => {
                            if transaction_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionParameters"));
                            }
                            transaction_parameters__ = map_.next_value()?;
                        }
                        GeneratedField::DetectionData => {
                            if detection_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("detectionData"));
                            }
                            detection_data__ = map_.next_value()?;
                        }
                        GeneratedField::Memo => {
                            if memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memo"));
                            }
                            memo__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionBody {
                    actions: actions__.unwrap_or_default(),
                    transaction_parameters: transaction_parameters__,
                    detection_data: detection_data__,
                    memo: memo__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionBodyView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.action_views.is_empty() {
            len += 1;
        }
        if self.transaction_parameters.is_some() {
            len += 1;
        }
        if self.detection_data.is_some() {
            len += 1;
        }
        if self.memo_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionBodyView", len)?;
        if !self.action_views.is_empty() {
            struct_ser.serialize_field("actionViews", &self.action_views)?;
        }
        if let Some(v) = self.transaction_parameters.as_ref() {
            struct_ser.serialize_field("transactionParameters", v)?;
        }
        if let Some(v) = self.detection_data.as_ref() {
            struct_ser.serialize_field("detectionData", v)?;
        }
        if let Some(v) = self.memo_view.as_ref() {
            struct_ser.serialize_field("memoView", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionBodyView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "action_views",
            "actionViews",
            "transaction_parameters",
            "transactionParameters",
            "detection_data",
            "detectionData",
            "memo_view",
            "memoView",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ActionViews,
            TransactionParameters,
            DetectionData,
            MemoView,
            __SkipField__,
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
                            "actionViews" | "action_views" => Ok(GeneratedField::ActionViews),
                            "transactionParameters" | "transaction_parameters" => Ok(GeneratedField::TransactionParameters),
                            "detectionData" | "detection_data" => Ok(GeneratedField::DetectionData),
                            "memoView" | "memo_view" => Ok(GeneratedField::MemoView),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionBodyView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionBodyView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionBodyView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action_views__ = None;
                let mut transaction_parameters__ = None;
                let mut detection_data__ = None;
                let mut memo_view__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ActionViews => {
                            if action_views__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionViews"));
                            }
                            action_views__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TransactionParameters => {
                            if transaction_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionParameters"));
                            }
                            transaction_parameters__ = map_.next_value()?;
                        }
                        GeneratedField::DetectionData => {
                            if detection_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("detectionData"));
                            }
                            detection_data__ = map_.next_value()?;
                        }
                        GeneratedField::MemoView => {
                            if memo_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memoView"));
                            }
                            memo_view__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionBodyView {
                    action_views: action_views__.unwrap_or_default(),
                    transaction_parameters: transaction_parameters__,
                    detection_data: detection_data__,
                    memo_view: memo_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionBodyView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionParameters {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.expiry_height != 0 {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionParameters", len)?;
        if self.expiry_height != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionParameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "expiry_height",
            "expiryHeight",
            "chain_id",
            "chainId",
            "fee",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExpiryHeight,
            ChainId,
            Fee,
            __SkipField__,
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
                            "expiryHeight" | "expiry_height" => Ok(GeneratedField::ExpiryHeight),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "fee" => Ok(GeneratedField::Fee),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionParameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionParameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionParameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut expiry_height__ = None;
                let mut chain_id__ = None;
                let mut fee__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ExpiryHeight => {
                            if expiry_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryHeight"));
                            }
                            expiry_height__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionParameters {
                    expiry_height: expiry_height__.unwrap_or_default(),
                    chain_id: chain_id__.unwrap_or_default(),
                    fee: fee__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionParameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPerspective {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.payload_keys.is_empty() {
            len += 1;
        }
        if !self.spend_nullifiers.is_empty() {
            len += 1;
        }
        if !self.advice_notes.is_empty() {
            len += 1;
        }
        if !self.address_views.is_empty() {
            len += 1;
        }
        if !self.denoms.is_empty() {
            len += 1;
        }
        if self.transaction_id.is_some() {
            len += 1;
        }
        if !self.prices.is_empty() {
            len += 1;
        }
        if !self.extended_metadata.is_empty() {
            len += 1;
        }
        if !self.creation_transaction_ids_by_nullifier.is_empty() {
            len += 1;
        }
        if !self.nullification_transaction_ids_by_commitment.is_empty() {
            len += 1;
        }
        if !self.batch_swap_output_data.is_empty() {
            len += 1;
        }
        if self.position_metadata_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionPerspective", len)?;
        if !self.payload_keys.is_empty() {
            struct_ser.serialize_field("payloadKeys", &self.payload_keys)?;
        }
        if !self.spend_nullifiers.is_empty() {
            struct_ser.serialize_field("spendNullifiers", &self.spend_nullifiers)?;
        }
        if !self.advice_notes.is_empty() {
            struct_ser.serialize_field("adviceNotes", &self.advice_notes)?;
        }
        if !self.address_views.is_empty() {
            struct_ser.serialize_field("addressViews", &self.address_views)?;
        }
        if !self.denoms.is_empty() {
            struct_ser.serialize_field("denoms", &self.denoms)?;
        }
        if let Some(v) = self.transaction_id.as_ref() {
            struct_ser.serialize_field("transactionId", v)?;
        }
        if !self.prices.is_empty() {
            struct_ser.serialize_field("prices", &self.prices)?;
        }
        if !self.extended_metadata.is_empty() {
            struct_ser.serialize_field("extendedMetadata", &self.extended_metadata)?;
        }
        if !self.creation_transaction_ids_by_nullifier.is_empty() {
            struct_ser.serialize_field("creationTransactionIdsByNullifier", &self.creation_transaction_ids_by_nullifier)?;
        }
        if !self.nullification_transaction_ids_by_commitment.is_empty() {
            struct_ser.serialize_field("nullificationTransactionIdsByCommitment", &self.nullification_transaction_ids_by_commitment)?;
        }
        if !self.batch_swap_output_data.is_empty() {
            struct_ser.serialize_field("batchSwapOutputData", &self.batch_swap_output_data)?;
        }
        if let Some(v) = self.position_metadata_key.as_ref() {
            struct_ser.serialize_field("positionMetadataKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPerspective {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "payload_keys",
            "payloadKeys",
            "spend_nullifiers",
            "spendNullifiers",
            "advice_notes",
            "adviceNotes",
            "address_views",
            "addressViews",
            "denoms",
            "transaction_id",
            "transactionId",
            "prices",
            "extended_metadata",
            "extendedMetadata",
            "creation_transaction_ids_by_nullifier",
            "creationTransactionIdsByNullifier",
            "nullification_transaction_ids_by_commitment",
            "nullificationTransactionIdsByCommitment",
            "batch_swap_output_data",
            "batchSwapOutputData",
            "position_metadata_key",
            "positionMetadataKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayloadKeys,
            SpendNullifiers,
            AdviceNotes,
            AddressViews,
            Denoms,
            TransactionId,
            Prices,
            ExtendedMetadata,
            CreationTransactionIdsByNullifier,
            NullificationTransactionIdsByCommitment,
            BatchSwapOutputData,
            PositionMetadataKey,
            __SkipField__,
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
                            "payloadKeys" | "payload_keys" => Ok(GeneratedField::PayloadKeys),
                            "spendNullifiers" | "spend_nullifiers" => Ok(GeneratedField::SpendNullifiers),
                            "adviceNotes" | "advice_notes" => Ok(GeneratedField::AdviceNotes),
                            "addressViews" | "address_views" => Ok(GeneratedField::AddressViews),
                            "denoms" => Ok(GeneratedField::Denoms),
                            "transactionId" | "transaction_id" => Ok(GeneratedField::TransactionId),
                            "prices" => Ok(GeneratedField::Prices),
                            "extendedMetadata" | "extended_metadata" => Ok(GeneratedField::ExtendedMetadata),
                            "creationTransactionIdsByNullifier" | "creation_transaction_ids_by_nullifier" => Ok(GeneratedField::CreationTransactionIdsByNullifier),
                            "nullificationTransactionIdsByCommitment" | "nullification_transaction_ids_by_commitment" => Ok(GeneratedField::NullificationTransactionIdsByCommitment),
                            "batchSwapOutputData" | "batch_swap_output_data" => Ok(GeneratedField::BatchSwapOutputData),
                            "positionMetadataKey" | "position_metadata_key" => Ok(GeneratedField::PositionMetadataKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPerspective;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionPerspective")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionPerspective, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload_keys__ = None;
                let mut spend_nullifiers__ = None;
                let mut advice_notes__ = None;
                let mut address_views__ = None;
                let mut denoms__ = None;
                let mut transaction_id__ = None;
                let mut prices__ = None;
                let mut extended_metadata__ = None;
                let mut creation_transaction_ids_by_nullifier__ = None;
                let mut nullification_transaction_ids_by_commitment__ = None;
                let mut batch_swap_output_data__ = None;
                let mut position_metadata_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PayloadKeys => {
                            if payload_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloadKeys"));
                            }
                            payload_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SpendNullifiers => {
                            if spend_nullifiers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendNullifiers"));
                            }
                            spend_nullifiers__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AdviceNotes => {
                            if advice_notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("adviceNotes"));
                            }
                            advice_notes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AddressViews => {
                            if address_views__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressViews"));
                            }
                            address_views__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Denoms => {
                            if denoms__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denoms"));
                            }
                            denoms__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TransactionId => {
                            if transaction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionId"));
                            }
                            transaction_id__ = map_.next_value()?;
                        }
                        GeneratedField::Prices => {
                            if prices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prices"));
                            }
                            prices__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExtendedMetadata => {
                            if extended_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extendedMetadata"));
                            }
                            extended_metadata__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreationTransactionIdsByNullifier => {
                            if creation_transaction_ids_by_nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("creationTransactionIdsByNullifier"));
                            }
                            creation_transaction_ids_by_nullifier__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NullificationTransactionIdsByCommitment => {
                            if nullification_transaction_ids_by_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullificationTransactionIdsByCommitment"));
                            }
                            nullification_transaction_ids_by_commitment__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BatchSwapOutputData => {
                            if batch_swap_output_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("batchSwapOutputData"));
                            }
                            batch_swap_output_data__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PositionMetadataKey => {
                            if position_metadata_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionMetadataKey"));
                            }
                            position_metadata_key__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionPerspective {
                    payload_keys: payload_keys__.unwrap_or_default(),
                    spend_nullifiers: spend_nullifiers__.unwrap_or_default(),
                    advice_notes: advice_notes__.unwrap_or_default(),
                    address_views: address_views__.unwrap_or_default(),
                    denoms: denoms__.unwrap_or_default(),
                    transaction_id: transaction_id__,
                    prices: prices__.unwrap_or_default(),
                    extended_metadata: extended_metadata__.unwrap_or_default(),
                    creation_transaction_ids_by_nullifier: creation_transaction_ids_by_nullifier__.unwrap_or_default(),
                    nullification_transaction_ids_by_commitment: nullification_transaction_ids_by_commitment__.unwrap_or_default(),
                    batch_swap_output_data: batch_swap_output_data__.unwrap_or_default(),
                    position_metadata_key: position_metadata_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionPerspective", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_perspective::CreationTransactionIdByNullifier {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.nullifier.is_some() {
            len += 1;
        }
        if self.transaction_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionPerspective.CreationTransactionIdByNullifier", len)?;
        if let Some(v) = self.nullifier.as_ref() {
            struct_ser.serialize_field("nullifier", v)?;
        }
        if let Some(v) = self.transaction_id.as_ref() {
            struct_ser.serialize_field("transactionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_perspective::CreationTransactionIdByNullifier {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "nullifier",
            "transaction_id",
            "transactionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Nullifier,
            TransactionId,
            __SkipField__,
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
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "transactionId" | "transaction_id" => Ok(GeneratedField::TransactionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_perspective::CreationTransactionIdByNullifier;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionPerspective.CreationTransactionIdByNullifier")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_perspective::CreationTransactionIdByNullifier, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut transaction_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map_.next_value()?;
                        }
                        GeneratedField::TransactionId => {
                            if transaction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionId"));
                            }
                            transaction_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_perspective::CreationTransactionIdByNullifier {
                    nullifier: nullifier__,
                    transaction_id: transaction_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionPerspective.CreationTransactionIdByNullifier", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_perspective::ExtendedMetadataById {
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
        if self.extended_metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionPerspective.ExtendedMetadataById", len)?;
        if let Some(v) = self.asset_id.as_ref() {
            struct_ser.serialize_field("assetId", v)?;
        }
        if let Some(v) = self.extended_metadata.as_ref() {
            struct_ser.serialize_field("extendedMetadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_perspective::ExtendedMetadataById {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "asset_id",
            "assetId",
            "extended_metadata",
            "extendedMetadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AssetId,
            ExtendedMetadata,
            __SkipField__,
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
                            "extendedMetadata" | "extended_metadata" => Ok(GeneratedField::ExtendedMetadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_perspective::ExtendedMetadataById;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionPerspective.ExtendedMetadataById")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_perspective::ExtendedMetadataById, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut asset_id__ = None;
                let mut extended_metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AssetId => {
                            if asset_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("assetId"));
                            }
                            asset_id__ = map_.next_value()?;
                        }
                        GeneratedField::ExtendedMetadata => {
                            if extended_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extendedMetadata"));
                            }
                            extended_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_perspective::ExtendedMetadataById {
                    asset_id: asset_id__,
                    extended_metadata: extended_metadata__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionPerspective.ExtendedMetadataById", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_perspective::NullificationTransactionIdByCommitment {
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
        if self.transaction_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionPerspective.NullificationTransactionIdByCommitment", len)?;
        if let Some(v) = self.commitment.as_ref() {
            struct_ser.serialize_field("commitment", v)?;
        }
        if let Some(v) = self.transaction_id.as_ref() {
            struct_ser.serialize_field("transactionId", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_perspective::NullificationTransactionIdByCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commitment",
            "transaction_id",
            "transactionId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commitment,
            TransactionId,
            __SkipField__,
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
                            "transactionId" | "transaction_id" => Ok(GeneratedField::TransactionId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_perspective::NullificationTransactionIdByCommitment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionPerspective.NullificationTransactionIdByCommitment")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_perspective::NullificationTransactionIdByCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commitment__ = None;
                let mut transaction_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map_.next_value()?;
                        }
                        GeneratedField::TransactionId => {
                            if transaction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionId"));
                            }
                            transaction_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_perspective::NullificationTransactionIdByCommitment {
                    commitment: commitment__,
                    transaction_id: transaction_id__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionPerspective.NullificationTransactionIdByCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.actions.is_empty() {
            len += 1;
        }
        if self.transaction_parameters.is_some() {
            len += 1;
        }
        if self.detection_data.is_some() {
            len += 1;
        }
        if self.memo.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionPlan", len)?;
        if !self.actions.is_empty() {
            struct_ser.serialize_field("actions", &self.actions)?;
        }
        if let Some(v) = self.transaction_parameters.as_ref() {
            struct_ser.serialize_field("transactionParameters", v)?;
        }
        if let Some(v) = self.detection_data.as_ref() {
            struct_ser.serialize_field("detectionData", v)?;
        }
        if let Some(v) = self.memo.as_ref() {
            struct_ser.serialize_field("memo", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "actions",
            "transaction_parameters",
            "transactionParameters",
            "detection_data",
            "detectionData",
            "memo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Actions,
            TransactionParameters,
            DetectionData,
            Memo,
            __SkipField__,
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
                            "actions" => Ok(GeneratedField::Actions),
                            "transactionParameters" | "transaction_parameters" => Ok(GeneratedField::TransactionParameters),
                            "detectionData" | "detection_data" => Ok(GeneratedField::DetectionData),
                            "memo" => Ok(GeneratedField::Memo),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionPlan")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut actions__ = None;
                let mut transaction_parameters__ = None;
                let mut detection_data__ = None;
                let mut memo__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Actions => {
                            if actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actions"));
                            }
                            actions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TransactionParameters => {
                            if transaction_parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionParameters"));
                            }
                            transaction_parameters__ = map_.next_value()?;
                        }
                        GeneratedField::DetectionData => {
                            if detection_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("detectionData"));
                            }
                            detection_data__ = map_.next_value()?;
                        }
                        GeneratedField::Memo => {
                            if memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memo"));
                            }
                            memo__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionPlan {
                    actions: actions__.unwrap_or_default(),
                    transaction_parameters: transaction_parameters__,
                    detection_data: detection_data__,
                    memo: memo__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionSummary {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.effects.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionSummary", len)?;
        if !self.effects.is_empty() {
            struct_ser.serialize_field("effects", &self.effects)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionSummary {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "effects",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Effects,
            __SkipField__,
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
                            "effects" => Ok(GeneratedField::Effects),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionSummary;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionSummary")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionSummary, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut effects__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Effects => {
                            if effects__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effects"));
                            }
                            effects__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionSummary {
                    effects: effects__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionSummary", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for transaction_summary::Effects {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.address.is_some() {
            len += 1;
        }
        if self.balance.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionSummary.Effects", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if let Some(v) = self.balance.as_ref() {
            struct_ser.serialize_field("balance", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for transaction_summary::Effects {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "balance",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Balance,
            __SkipField__,
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
                            "address" => Ok(GeneratedField::Address),
                            "balance" => Ok(GeneratedField::Balance),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = transaction_summary::Effects;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionSummary.Effects")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<transaction_summary::Effects, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut balance__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map_.next_value()?;
                        }
                        GeneratedField::Balance => {
                            if balance__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balance"));
                            }
                            balance__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(transaction_summary::Effects {
                    address: address__,
                    balance: balance__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionSummary.Effects", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.body_view.is_some() {
            len += 1;
        }
        if self.binding_sig.is_some() {
            len += 1;
        }
        if self.anchor.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.TransactionView", len)?;
        if let Some(v) = self.body_view.as_ref() {
            struct_ser.serialize_field("bodyView", v)?;
        }
        if let Some(v) = self.binding_sig.as_ref() {
            struct_ser.serialize_field("bindingSig", v)?;
        }
        if let Some(v) = self.anchor.as_ref() {
            struct_ser.serialize_field("anchor", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TransactionView {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body_view",
            "bodyView",
            "binding_sig",
            "bindingSig",
            "anchor",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BodyView,
            BindingSig,
            Anchor,
            __SkipField__,
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
                            "bodyView" | "body_view" => Ok(GeneratedField::BodyView),
                            "bindingSig" | "binding_sig" => Ok(GeneratedField::BindingSig),
                            "anchor" => Ok(GeneratedField::Anchor),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.TransactionView")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TransactionView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body_view__ = None;
                let mut binding_sig__ = None;
                let mut anchor__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BodyView => {
                            if body_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bodyView"));
                            }
                            body_view__ = map_.next_value()?;
                        }
                        GeneratedField::BindingSig => {
                            if binding_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bindingSig"));
                            }
                            binding_sig__ = map_.next_value()?;
                        }
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TransactionView {
                    body_view: body_view__,
                    binding_sig: binding_sig__,
                    anchor: anchor__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.TransactionView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WitnessData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.anchor.is_some() {
            len += 1;
        }
        if !self.state_commitment_proofs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1.WitnessData", len)?;
        if let Some(v) = self.anchor.as_ref() {
            struct_ser.serialize_field("anchor", v)?;
        }
        if !self.state_commitment_proofs.is_empty() {
            struct_ser.serialize_field("stateCommitmentProofs", &self.state_commitment_proofs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WitnessData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "anchor",
            "state_commitment_proofs",
            "stateCommitmentProofs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Anchor,
            StateCommitmentProofs,
            __SkipField__,
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
                            "anchor" => Ok(GeneratedField::Anchor),
                            "stateCommitmentProofs" | "state_commitment_proofs" => Ok(GeneratedField::StateCommitmentProofs),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WitnessData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1.WitnessData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WitnessData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut anchor__ = None;
                let mut state_commitment_proofs__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map_.next_value()?;
                        }
                        GeneratedField::StateCommitmentProofs => {
                            if state_commitment_proofs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stateCommitmentProofs"));
                            }
                            state_commitment_proofs__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WitnessData {
                    anchor: anchor__,
                    state_commitment_proofs: state_commitment_proofs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1.WitnessData", FIELDS, GeneratedVisitor)
    }
}
