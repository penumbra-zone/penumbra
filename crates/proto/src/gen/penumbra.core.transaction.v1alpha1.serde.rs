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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.Action", len)?;
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
                action::Action::IbcAction(v) => {
                    struct_ser.serialize_field("ibcAction", v)?;
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
                action::Action::DaoSpend(v) => {
                    struct_ser.serialize_field("daoSpend", v)?;
                }
                action::Action::DaoOutput(v) => {
                    struct_ser.serialize_field("daoOutput", v)?;
                }
                action::Action::DaoDeposit(v) => {
                    struct_ser.serialize_field("daoDeposit", v)?;
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
            "ibc_action",
            "ibcAction",
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
            "dao_spend",
            "daoSpend",
            "dao_output",
            "daoOutput",
            "dao_deposit",
            "daoDeposit",
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
            IbcAction,
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
            DaoSpend,
            DaoOutput,
            DaoDeposit,
            Ics20Withdrawal,
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
                            "ibcAction" | "ibc_action" => Ok(GeneratedField::IbcAction),
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
                            "daoSpend" | "dao_spend" => Ok(GeneratedField::DaoSpend),
                            "daoOutput" | "dao_output" => Ok(GeneratedField::DaoOutput),
                            "daoDeposit" | "dao_deposit" => Ok(GeneratedField::DaoDeposit),
                            "ics20Withdrawal" | "ics20_withdrawal" => Ok(GeneratedField::Ics20Withdrawal),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.Action")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Action, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::Spend)
;
                        }
                        GeneratedField::Output => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::Output)
;
                        }
                        GeneratedField::Swap => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::Swap)
;
                        }
                        GeneratedField::SwapClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::SwapClaim)
;
                        }
                        GeneratedField::ValidatorDefinition => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::ValidatorDefinition)
;
                        }
                        GeneratedField::IbcAction => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcAction"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::IbcAction)
;
                        }
                        GeneratedField::ProposalSubmit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSubmit"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::ProposalSubmit)
;
                        }
                        GeneratedField::ProposalWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalWithdraw"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::ProposalWithdraw)
;
                        }
                        GeneratedField::ValidatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::ValidatorVote)
;
                        }
                        GeneratedField::DelegatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::DelegatorVote)
;
                        }
                        GeneratedField::ProposalDepositClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::ProposalDepositClaim)
;
                        }
                        GeneratedField::PositionOpen => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpen"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionOpen)
;
                        }
                        GeneratedField::PositionClose => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionClose"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionClose)
;
                        }
                        GeneratedField::PositionWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraw"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionWithdraw)
;
                        }
                        GeneratedField::PositionRewardClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionRewardClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::PositionRewardClaim)
;
                        }
                        GeneratedField::Delegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegate"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::Delegate)
;
                        }
                        GeneratedField::Undelegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegate"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::Undelegate)
;
                        }
                        GeneratedField::UndelegateClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::UndelegateClaim)
;
                        }
                        GeneratedField::DaoSpend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoSpend"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::DaoSpend)
;
                        }
                        GeneratedField::DaoOutput => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoOutput"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::DaoOutput)
;
                        }
                        GeneratedField::DaoDeposit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoDeposit"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::DaoDeposit)
;
                        }
                        GeneratedField::Ics20Withdrawal => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Withdrawal"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action::Action::Ics20Withdrawal)
;
                        }
                    }
                }
                Ok(Action {
                    action: action__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.Action", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.ActionPlan", len)?;
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
                action_plan::Action::IbcAction(v) => {
                    struct_ser.serialize_field("ibcAction", v)?;
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
                action_plan::Action::Withdrawal(v) => {
                    struct_ser.serialize_field("withdrawal", v)?;
                }
                action_plan::Action::PositionOpen(v) => {
                    struct_ser.serialize_field("positionOpen", v)?;
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
                action_plan::Action::DaoSpend(v) => {
                    struct_ser.serialize_field("daoSpend", v)?;
                }
                action_plan::Action::DaoOutput(v) => {
                    struct_ser.serialize_field("daoOutput", v)?;
                }
                action_plan::Action::DaoDeposit(v) => {
                    struct_ser.serialize_field("daoDeposit", v)?;
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
            "ibc_action",
            "ibcAction",
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
            "withdrawal",
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
            "dao_spend",
            "daoSpend",
            "dao_output",
            "daoOutput",
            "dao_deposit",
            "daoDeposit",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
            Output,
            Swap,
            SwapClaim,
            ValidatorDefinition,
            IbcAction,
            ProposalSubmit,
            ProposalWithdraw,
            ValidatorVote,
            DelegatorVote,
            ProposalDepositClaim,
            Withdrawal,
            PositionOpen,
            PositionClose,
            PositionWithdraw,
            PositionRewardClaim,
            Delegate,
            Undelegate,
            UndelegateClaim,
            DaoSpend,
            DaoOutput,
            DaoDeposit,
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
                            "ibcAction" | "ibc_action" => Ok(GeneratedField::IbcAction),
                            "proposalSubmit" | "proposal_submit" => Ok(GeneratedField::ProposalSubmit),
                            "proposalWithdraw" | "proposal_withdraw" => Ok(GeneratedField::ProposalWithdraw),
                            "validatorVote" | "validator_vote" => Ok(GeneratedField::ValidatorVote),
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            "proposalDepositClaim" | "proposal_deposit_claim" => Ok(GeneratedField::ProposalDepositClaim),
                            "withdrawal" => Ok(GeneratedField::Withdrawal),
                            "positionOpen" | "position_open" => Ok(GeneratedField::PositionOpen),
                            "positionClose" | "position_close" => Ok(GeneratedField::PositionClose),
                            "positionWithdraw" | "position_withdraw" => Ok(GeneratedField::PositionWithdraw),
                            "positionRewardClaim" | "position_reward_claim" => Ok(GeneratedField::PositionRewardClaim),
                            "delegate" => Ok(GeneratedField::Delegate),
                            "undelegate" => Ok(GeneratedField::Undelegate),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "daoSpend" | "dao_spend" => Ok(GeneratedField::DaoSpend),
                            "daoOutput" | "dao_output" => Ok(GeneratedField::DaoOutput),
                            "daoDeposit" | "dao_deposit" => Ok(GeneratedField::DaoDeposit),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.ActionPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActionPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Spend)
;
                        }
                        GeneratedField::Output => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Output)
;
                        }
                        GeneratedField::Swap => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Swap)
;
                        }
                        GeneratedField::SwapClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::SwapClaim)
;
                        }
                        GeneratedField::ValidatorDefinition => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ValidatorDefinition)
;
                        }
                        GeneratedField::IbcAction => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcAction"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::IbcAction)
;
                        }
                        GeneratedField::ProposalSubmit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSubmit"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ProposalSubmit)
;
                        }
                        GeneratedField::ProposalWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalWithdraw"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ProposalWithdraw)
;
                        }
                        GeneratedField::ValidatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ValidatorVote)
;
                        }
                        GeneratedField::DelegatorVote => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::DelegatorVote)
;
                        }
                        GeneratedField::ProposalDepositClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::ProposalDepositClaim)
;
                        }
                        GeneratedField::Withdrawal => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("withdrawal"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Withdrawal)
;
                        }
                        GeneratedField::PositionOpen => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpen"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionOpen)
;
                        }
                        GeneratedField::PositionClose => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionClose"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionClose)
;
                        }
                        GeneratedField::PositionWithdraw => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraw"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionWithdraw)
;
                        }
                        GeneratedField::PositionRewardClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionRewardClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::PositionRewardClaim)
;
                        }
                        GeneratedField::Delegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegate"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Delegate)
;
                        }
                        GeneratedField::Undelegate => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegate"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::Undelegate)
;
                        }
                        GeneratedField::UndelegateClaim => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::UndelegateClaim)
;
                        }
                        GeneratedField::DaoSpend => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoSpend"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::DaoSpend)
;
                        }
                        GeneratedField::DaoOutput => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoOutput"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::DaoOutput)
;
                        }
                        GeneratedField::DaoDeposit => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoDeposit"));
                            }
                            action__ = map.next_value::<::std::option::Option<_>>()?.map(action_plan::Action::DaoDeposit)
;
                        }
                    }
                }
                Ok(ActionPlan {
                    action: action__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.ActionPlan", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.ActionView", len)?;
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
                action_view::ActionView::ValidatorDefinition(v) => {
                    struct_ser.serialize_field("validatorDefinition", v)?;
                }
                action_view::ActionView::IbcAction(v) => {
                    struct_ser.serialize_field("ibcAction", v)?;
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
                action_view::ActionView::DelegatorVote(v) => {
                    struct_ser.serialize_field("delegatorVote", v)?;
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
                action_view::ActionView::DaoSpend(v) => {
                    struct_ser.serialize_field("daoSpend", v)?;
                }
                action_view::ActionView::DaoOutput(v) => {
                    struct_ser.serialize_field("daoOutput", v)?;
                }
                action_view::ActionView::DaoDeposit(v) => {
                    struct_ser.serialize_field("daoDeposit", v)?;
                }
                action_view::ActionView::UndelegateClaim(v) => {
                    struct_ser.serialize_field("undelegateClaim", v)?;
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
            "validator_definition",
            "validatorDefinition",
            "ibc_action",
            "ibcAction",
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
            "dao_spend",
            "daoSpend",
            "dao_output",
            "daoOutput",
            "dao_deposit",
            "daoDeposit",
            "undelegate_claim",
            "undelegateClaim",
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
            IbcAction,
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
            DaoSpend,
            DaoOutput,
            DaoDeposit,
            UndelegateClaim,
            Ics20Withdrawal,
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
                            "ibcAction" | "ibc_action" => Ok(GeneratedField::IbcAction),
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
                            "daoSpend" | "dao_spend" => Ok(GeneratedField::DaoSpend),
                            "daoOutput" | "dao_output" => Ok(GeneratedField::DaoOutput),
                            "daoDeposit" | "dao_deposit" => Ok(GeneratedField::DaoDeposit),
                            "undelegateClaim" | "undelegate_claim" => Ok(GeneratedField::UndelegateClaim),
                            "ics20Withdrawal" | "ics20_withdrawal" => Ok(GeneratedField::Ics20Withdrawal),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.ActionView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ActionView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Spend)
;
                        }
                        GeneratedField::Output => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Output)
;
                        }
                        GeneratedField::Swap => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swap"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Swap)
;
                        }
                        GeneratedField::SwapClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swapClaim"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::SwapClaim)
;
                        }
                        GeneratedField::ValidatorDefinition => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorDefinition"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ValidatorDefinition)
;
                        }
                        GeneratedField::IbcAction => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ibcAction"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::IbcAction)
;
                        }
                        GeneratedField::ProposalSubmit => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSubmit"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ProposalSubmit)
;
                        }
                        GeneratedField::ProposalWithdraw => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalWithdraw"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ProposalWithdraw)
;
                        }
                        GeneratedField::ValidatorVote => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validatorVote"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ValidatorVote)
;
                        }
                        GeneratedField::DelegatorVote => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::DelegatorVote)
;
                        }
                        GeneratedField::ProposalDepositClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalDepositClaim"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::ProposalDepositClaim)
;
                        }
                        GeneratedField::PositionOpen => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionOpen"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionOpen)
;
                        }
                        GeneratedField::PositionClose => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionClose"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionClose)
;
                        }
                        GeneratedField::PositionWithdraw => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionWithdraw"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionWithdraw)
;
                        }
                        GeneratedField::PositionRewardClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionRewardClaim"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::PositionRewardClaim)
;
                        }
                        GeneratedField::Delegate => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegate"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Delegate)
;
                        }
                        GeneratedField::Undelegate => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegate"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Undelegate)
;
                        }
                        GeneratedField::DaoSpend => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoSpend"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::DaoSpend)
;
                        }
                        GeneratedField::DaoOutput => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoOutput"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::DaoOutput)
;
                        }
                        GeneratedField::DaoDeposit => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("daoDeposit"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::DaoDeposit)
;
                        }
                        GeneratedField::UndelegateClaim => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("undelegateClaim"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::UndelegateClaim)
;
                        }
                        GeneratedField::Ics20Withdrawal => {
                            if action_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ics20Withdrawal"));
                            }
                            action_view__ = map.next_value::<::std::option::Option<_>>()?.map(action_view::ActionView::Ics20Withdrawal)
;
                        }
                    }
                }
                Ok(ActionView {
                    action_view: action_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.ActionView", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.AuthorizationData", len)?;
        if let Some(v) = self.effect_hash.as_ref() {
            struct_ser.serialize_field("effectHash", v)?;
        }
        if !self.spend_auths.is_empty() {
            struct_ser.serialize_field("spendAuths", &self.spend_auths)?;
        }
        if !self.delegator_vote_auths.is_empty() {
            struct_ser.serialize_field("delegatorVoteAuths", &self.delegator_vote_auths)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EffectHash,
            SpendAuths,
            DelegatorVoteAuths,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.AuthorizationData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthorizationData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut effect_hash__ = None;
                let mut spend_auths__ = None;
                let mut delegator_vote_auths__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::EffectHash => {
                            if effect_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effectHash"));
                            }
                            effect_hash__ = map.next_value()?;
                        }
                        GeneratedField::SpendAuths => {
                            if spend_auths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendAuths"));
                            }
                            spend_auths__ = Some(map.next_value()?);
                        }
                        GeneratedField::DelegatorVoteAuths => {
                            if delegator_vote_auths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVoteAuths"));
                            }
                            delegator_vote_auths__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AuthorizationData {
                    effect_hash: effect_hash__,
                    spend_auths: spend_auths__.unwrap_or_default(),
                    delegator_vote_auths: delegator_vote_auths__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.AuthorizationData", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.CluePlan", len)?;
        if let Some(v) = self.address.as_ref() {
            struct_ser.serialize_field("address", v)?;
        }
        if !self.rseed.is_empty() {
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        if self.precision_bits != 0 {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.CluePlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CluePlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut rseed__ = None;
                let mut precision_bits__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = map.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::PrecisionBits => {
                            if precision_bits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("precisionBits"));
                            }
                            precision_bits__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
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
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.CluePlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DelegatorVoteView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegator_vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.DelegatorVoteView", len)?;
        if let Some(v) = self.delegator_vote.as_ref() {
            match v {
                delegator_vote_view::DelegatorVote::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                delegator_vote_view::DelegatorVote::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DelegatorVoteView {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DelegatorVoteView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.DelegatorVoteView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DelegatorVoteView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_vote__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            delegator_vote__ = map.next_value::<::std::option::Option<_>>()?.map(delegator_vote_view::DelegatorVote::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            delegator_vote__ = map.next_value::<::std::option::Option<_>>()?.map(delegator_vote_view::DelegatorVote::Opaque)
;
                        }
                    }
                }
                Ok(DelegatorVoteView {
                    delegator_vote: delegator_vote__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.DelegatorVoteView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for delegator_vote_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegator_vote.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.DelegatorVoteView.Opaque", len)?;
        if let Some(v) = self.delegator_vote.as_ref() {
            struct_ser.serialize_field("delegatorVote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for delegator_vote_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_vote",
            "delegatorVote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorVote,
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
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = delegator_vote_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.DelegatorVoteView.Opaque")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<delegator_vote_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_vote__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = map.next_value()?;
                        }
                    }
                }
                Ok(delegator_vote_view::Opaque {
                    delegator_vote: delegator_vote__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.DelegatorVoteView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for delegator_vote_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.delegator_vote.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.DelegatorVoteView.Visible", len)?;
        if let Some(v) = self.delegator_vote.as_ref() {
            struct_ser.serialize_field("delegatorVote", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for delegator_vote_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "delegator_vote",
            "delegatorVote",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DelegatorVote,
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
                            "delegatorVote" | "delegator_vote" => Ok(GeneratedField::DelegatorVote),
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
            type Value = delegator_vote_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.DelegatorVoteView.Visible")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<delegator_vote_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut delegator_vote__ = None;
                let mut note__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::DelegatorVote => {
                            if delegator_vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delegatorVote"));
                            }
                            delegator_vote__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                    }
                }
                Ok(delegator_vote_view::Visible {
                    delegator_vote: delegator_vote__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.DelegatorVoteView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EffectHash {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.EffectHash", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EffectHash {
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
            type Value = EffectHash;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.EffectHash")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EffectHash, V::Error>
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
                Ok(EffectHash {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.EffectHash", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Id {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.hash.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.Id", len)?;
        if !self.hash.is_empty() {
            struct_ser.serialize_field("hash", pbjson::private::base64::encode(&self.hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Id {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "hash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Hash,
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
                            "hash" => Ok(GeneratedField::Hash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.Id")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Id, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut hash__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Hash => {
                            if hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hash"));
                            }
                            hash__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Id {
                    hash: hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.Id", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.MemoCiphertext", len)?;
        if !self.inner.is_empty() {
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
            type Value = MemoCiphertext;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.MemoCiphertext")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MemoCiphertext, V::Error>
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
                Ok(MemoCiphertext {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.MemoCiphertext", FIELDS, GeneratedVisitor)
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
        if self.sender.is_some() {
            len += 1;
        }
        if !self.text.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.MemoPlaintext", len)?;
        if let Some(v) = self.sender.as_ref() {
            struct_ser.serialize_field("sender", v)?;
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
            "sender",
            "text",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Sender,
            Text,
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
                            "sender" => Ok(GeneratedField::Sender),
                            "text" => Ok(GeneratedField::Text),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.MemoPlaintext")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MemoPlaintext, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sender__ = None;
                let mut text__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Sender => {
                            if sender__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sender"));
                            }
                            sender__ = map.next_value()?;
                        }
                        GeneratedField::Text => {
                            if text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            text__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MemoPlaintext {
                    sender: sender__,
                    text: text__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.MemoPlaintext", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.MemoPlan", len)?;
        if let Some(v) = self.plaintext.as_ref() {
            struct_ser.serialize_field("plaintext", v)?;
        }
        if !self.key.is_empty() {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.MemoPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MemoPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut plaintext__ = None;
                let mut key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Plaintext => {
                            if plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plaintext"));
                            }
                            plaintext__ = map.next_value()?;
                        }
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MemoPlan {
                    plaintext: plaintext__,
                    key: key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.MemoPlan", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.MemoView", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.MemoView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MemoView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut memo_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if memo_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            memo_view__ = map.next_value::<::std::option::Option<_>>()?.map(memo_view::MemoView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if memo_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            memo_view__ = map.next_value::<::std::option::Option<_>>()?.map(memo_view::MemoView::Opaque)
;
                        }
                    }
                }
                Ok(MemoView {
                    memo_view: memo_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.MemoView", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.MemoView.Opaque", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.MemoView.Opaque")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<memo_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ciphertext__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Ciphertext => {
                            if ciphertext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ciphertext"));
                            }
                            ciphertext__ = map.next_value()?;
                        }
                    }
                }
                Ok(memo_view::Opaque {
                    ciphertext: ciphertext__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.MemoView.Opaque", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.MemoView.Visible", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.MemoView.Visible")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<memo_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut ciphertext__ = None;
                let mut plaintext__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Ciphertext => {
                            if ciphertext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ciphertext"));
                            }
                            ciphertext__ = map.next_value()?;
                        }
                        GeneratedField::Plaintext => {
                            if plaintext__.is_some() {
                                return Err(serde::de::Error::duplicate_field("plaintext"));
                            }
                            plaintext__ = map.next_value()?;
                        }
                    }
                }
                Ok(memo_view::Visible {
                    ciphertext: ciphertext__,
                    plaintext: plaintext__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.MemoView.Visible", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.NullifierWithNote", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.NullifierWithNote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<NullifierWithNote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut nullifier__ = None;
                let mut note__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                    }
                }
                Ok(NullifierWithNote {
                    nullifier: nullifier__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.NullifierWithNote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Output {
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
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.Output", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Output {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            Proof,
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
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Output;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.Output")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Output, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map.next_value()?;
                        }
                    }
                }
                Ok(Output {
                    body: body__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.Output", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note_payload.is_some() {
            len += 1;
        }
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if !self.wrapped_memo_key.is_empty() {
            len += 1;
        }
        if !self.ovk_wrapped_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.OutputBody", len)?;
        if let Some(v) = self.note_payload.as_ref() {
            struct_ser.serialize_field("notePayload", v)?;
        }
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if !self.wrapped_memo_key.is_empty() {
            struct_ser.serialize_field("wrappedMemoKey", pbjson::private::base64::encode(&self.wrapped_memo_key).as_str())?;
        }
        if !self.ovk_wrapped_key.is_empty() {
            struct_ser.serialize_field("ovkWrappedKey", pbjson::private::base64::encode(&self.ovk_wrapped_key).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note_payload",
            "notePayload",
            "balance_commitment",
            "balanceCommitment",
            "wrapped_memo_key",
            "wrappedMemoKey",
            "ovk_wrapped_key",
            "ovkWrappedKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            NotePayload,
            BalanceCommitment,
            WrappedMemoKey,
            OvkWrappedKey,
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
                            "notePayload" | "note_payload" => Ok(GeneratedField::NotePayload),
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "wrappedMemoKey" | "wrapped_memo_key" => Ok(GeneratedField::WrappedMemoKey),
                            "ovkWrappedKey" | "ovk_wrapped_key" => Ok(GeneratedField::OvkWrappedKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.OutputBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OutputBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note_payload__ = None;
                let mut balance_commitment__ = None;
                let mut wrapped_memo_key__ = None;
                let mut ovk_wrapped_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::NotePayload => {
                            if note_payload__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notePayload"));
                            }
                            note_payload__ = map.next_value()?;
                        }
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map.next_value()?;
                        }
                        GeneratedField::WrappedMemoKey => {
                            if wrapped_memo_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wrappedMemoKey"));
                            }
                            wrapped_memo_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::OvkWrappedKey => {
                            if ovk_wrapped_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ovkWrappedKey"));
                            }
                            ovk_wrapped_key__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(OutputBody {
                    note_payload: note_payload__,
                    balance_commitment: balance_commitment__,
                    wrapped_memo_key: wrapped_memo_key__.unwrap_or_default(),
                    ovk_wrapped_key: ovk_wrapped_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.OutputBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        if self.dest_address.is_some() {
            len += 1;
        }
        if !self.rseed.is_empty() {
            len += 1;
        }
        if !self.value_blinding.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.OutputPlan", len)?;
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        if let Some(v) = self.dest_address.as_ref() {
            struct_ser.serialize_field("destAddress", v)?;
        }
        if !self.rseed.is_empty() {
            struct_ser.serialize_field("rseed", pbjson::private::base64::encode(&self.rseed).as_str())?;
        }
        if !self.value_blinding.is_empty() {
            struct_ser.serialize_field("valueBlinding", pbjson::private::base64::encode(&self.value_blinding).as_str())?;
        }
        if !self.proof_blinding_r.is_empty() {
            struct_ser.serialize_field("proofBlindingR", pbjson::private::base64::encode(&self.proof_blinding_r).as_str())?;
        }
        if !self.proof_blinding_s.is_empty() {
            struct_ser.serialize_field("proofBlindingS", pbjson::private::base64::encode(&self.proof_blinding_s).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "value",
            "dest_address",
            "destAddress",
            "rseed",
            "value_blinding",
            "valueBlinding",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Value,
            DestAddress,
            Rseed,
            ValueBlinding,
            ProofBlindingR,
            ProofBlindingS,
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
                            "value" => Ok(GeneratedField::Value),
                            "destAddress" | "dest_address" => Ok(GeneratedField::DestAddress),
                            "rseed" => Ok(GeneratedField::Rseed),
                            "valueBlinding" | "value_blinding" => Ok(GeneratedField::ValueBlinding),
                            "proofBlindingR" | "proof_blinding_r" => Ok(GeneratedField::ProofBlindingR),
                            "proofBlindingS" | "proof_blinding_s" => Ok(GeneratedField::ProofBlindingS),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.OutputPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OutputPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                let mut dest_address__ = None;
                let mut rseed__ = None;
                let mut value_blinding__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map.next_value()?;
                        }
                        GeneratedField::DestAddress => {
                            if dest_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("destAddress"));
                            }
                            dest_address__ = map.next_value()?;
                        }
                        GeneratedField::Rseed => {
                            if rseed__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rseed"));
                            }
                            rseed__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValueBlinding => {
                            if value_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueBlinding"));
                            }
                            value_blinding__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingR => {
                            if proof_blinding_r__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingR"));
                            }
                            proof_blinding_r__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingS => {
                            if proof_blinding_s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingS"));
                            }
                            proof_blinding_s__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(OutputPlan {
                    value: value__,
                    dest_address: dest_address__,
                    rseed: rseed__.unwrap_or_default(),
                    value_blinding: value_blinding__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.OutputPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.OutputView", len)?;
        if let Some(v) = self.output_view.as_ref() {
            match v {
                output_view::OutputView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                output_view::OutputView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputView {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.OutputView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<OutputView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if output_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            output_view__ = map.next_value::<::std::option::Option<_>>()?.map(output_view::OutputView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if output_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            output_view__ = map.next_value::<::std::option::Option<_>>()?.map(output_view::OutputView::Opaque)
;
                        }
                    }
                }
                Ok(OutputView {
                    output_view: output_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.OutputView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for output_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.OutputView.Opaque", len)?;
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for output_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "output",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Output,
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
                            "output" => Ok(GeneratedField::Output),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = output_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.OutputView.Opaque")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<output_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map.next_value()?;
                        }
                    }
                }
                Ok(output_view::Opaque {
                    output: output__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.OutputView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for output_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        if self.payload_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.OutputView.Visible", len)?;
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if let Some(v) = self.payload_key.as_ref() {
            struct_ser.serialize_field("payloadKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for output_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "output",
            "note",
            "payload_key",
            "payloadKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Output,
            Note,
            PayloadKey,
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
                            "output" => Ok(GeneratedField::Output),
                            "note" => Ok(GeneratedField::Note),
                            "payloadKey" | "payload_key" => Ok(GeneratedField::PayloadKey),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = output_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.OutputView.Visible")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<output_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output__ = None;
                let mut note__ = None;
                let mut payload_key__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                        GeneratedField::PayloadKey => {
                            if payload_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloadKey"));
                            }
                            payload_key__ = map.next_value()?;
                        }
                    }
                }
                Ok(output_view::Visible {
                    output: output__,
                    note: note__,
                    payload_key: payload_key__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.OutputView.Visible", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PayloadKey {
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.PayloadKey", len)?;
        if !self.inner.is_empty() {
            struct_ser.serialize_field("inner", pbjson::private::base64::encode(&self.inner).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PayloadKey {
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
            type Value = PayloadKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.PayloadKey")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PayloadKey, V::Error>
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
                Ok(PayloadKey {
                    inner: inner__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.PayloadKey", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.PayloadKeyWithCommitment", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.PayloadKeyWithCommitment")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PayloadKeyWithCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload_key__ = None;
                let mut commitment__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PayloadKey => {
                            if payload_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloadKey"));
                            }
                            payload_key__ = map.next_value()?;
                        }
                        GeneratedField::Commitment => {
                            if commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitment"));
                            }
                            commitment__ = map.next_value()?;
                        }
                    }
                }
                Ok(PayloadKeyWithCommitment {
                    payload_key: payload_key__,
                    commitment: commitment__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.PayloadKeyWithCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Spend {
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
        if self.auth_sig.is_some() {
            len += 1;
        }
        if self.proof.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.Spend", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if let Some(v) = self.auth_sig.as_ref() {
            struct_ser.serialize_field("authSig", v)?;
        }
        if let Some(v) = self.proof.as_ref() {
            struct_ser.serialize_field("proof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Spend {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "body",
            "auth_sig",
            "authSig",
            "proof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Body,
            AuthSig,
            Proof,
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
                            "authSig" | "auth_sig" => Ok(GeneratedField::AuthSig),
                            "proof" => Ok(GeneratedField::Proof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Spend;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.Spend")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Spend, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut auth_sig__ = None;
                let mut proof__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::AuthSig => {
                            if auth_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authSig"));
                            }
                            auth_sig__ = map.next_value()?;
                        }
                        GeneratedField::Proof => {
                            if proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proof"));
                            }
                            proof__ = map.next_value()?;
                        }
                    }
                }
                Ok(Spend {
                    body: body__,
                    auth_sig: auth_sig__,
                    proof: proof__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.Spend", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.balance_commitment.is_some() {
            len += 1;
        }
        if !self.nullifier.is_empty() {
            len += 1;
        }
        if !self.rk.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.SpendBody", len)?;
        if let Some(v) = self.balance_commitment.as_ref() {
            struct_ser.serialize_field("balanceCommitment", v)?;
        }
        if !self.nullifier.is_empty() {
            struct_ser.serialize_field("nullifier", pbjson::private::base64::encode(&self.nullifier).as_str())?;
        }
        if !self.rk.is_empty() {
            struct_ser.serialize_field("rk", pbjson::private::base64::encode(&self.rk).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "balance_commitment",
            "balanceCommitment",
            "nullifier",
            "rk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BalanceCommitment,
            Nullifier,
            Rk,
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
                            "balanceCommitment" | "balance_commitment" => Ok(GeneratedField::BalanceCommitment),
                            "nullifier" => Ok(GeneratedField::Nullifier),
                            "rk" => Ok(GeneratedField::Rk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendBody;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.SpendBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut balance_commitment__ = None;
                let mut nullifier__ = None;
                let mut rk__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BalanceCommitment => {
                            if balance_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("balanceCommitment"));
                            }
                            balance_commitment__ = map.next_value()?;
                        }
                        GeneratedField::Nullifier => {
                            if nullifier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nullifier"));
                            }
                            nullifier__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Rk => {
                            if rk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rk"));
                            }
                            rk__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SpendBody {
                    balance_commitment: balance_commitment__,
                    nullifier: nullifier__.unwrap_or_default(),
                    rk: rk__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.SpendBody", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendPlan {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.note.is_some() {
            len += 1;
        }
        if self.position != 0 {
            len += 1;
        }
        if !self.randomizer.is_empty() {
            len += 1;
        }
        if !self.value_blinding.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_r.is_empty() {
            len += 1;
        }
        if !self.proof_blinding_s.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.SpendPlan", len)?;
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        if self.position != 0 {
            struct_ser.serialize_field("position", ToString::to_string(&self.position).as_str())?;
        }
        if !self.randomizer.is_empty() {
            struct_ser.serialize_field("randomizer", pbjson::private::base64::encode(&self.randomizer).as_str())?;
        }
        if !self.value_blinding.is_empty() {
            struct_ser.serialize_field("valueBlinding", pbjson::private::base64::encode(&self.value_blinding).as_str())?;
        }
        if !self.proof_blinding_r.is_empty() {
            struct_ser.serialize_field("proofBlindingR", pbjson::private::base64::encode(&self.proof_blinding_r).as_str())?;
        }
        if !self.proof_blinding_s.is_empty() {
            struct_ser.serialize_field("proofBlindingS", pbjson::private::base64::encode(&self.proof_blinding_s).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendPlan {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "note",
            "position",
            "randomizer",
            "value_blinding",
            "valueBlinding",
            "proof_blinding_r",
            "proofBlindingR",
            "proof_blinding_s",
            "proofBlindingS",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Note,
            Position,
            Randomizer,
            ValueBlinding,
            ProofBlindingR,
            ProofBlindingS,
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
                            "note" => Ok(GeneratedField::Note),
                            "position" => Ok(GeneratedField::Position),
                            "randomizer" => Ok(GeneratedField::Randomizer),
                            "valueBlinding" | "value_blinding" => Ok(GeneratedField::ValueBlinding),
                            "proofBlindingR" | "proof_blinding_r" => Ok(GeneratedField::ProofBlindingR),
                            "proofBlindingS" | "proof_blinding_s" => Ok(GeneratedField::ProofBlindingS),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendPlan;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.SpendPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut note__ = None;
                let mut position__ = None;
                let mut randomizer__ = None;
                let mut value_blinding__ = None;
                let mut proof_blinding_r__ = None;
                let mut proof_blinding_s__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Randomizer => {
                            if randomizer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("randomizer"));
                            }
                            randomizer__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ValueBlinding => {
                            if value_blinding__.is_some() {
                                return Err(serde::de::Error::duplicate_field("valueBlinding"));
                            }
                            value_blinding__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingR => {
                            if proof_blinding_r__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingR"));
                            }
                            proof_blinding_r__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ProofBlindingS => {
                            if proof_blinding_s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proofBlindingS"));
                            }
                            proof_blinding_s__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SpendPlan {
                    note: note__,
                    position: position__.unwrap_or_default(),
                    randomizer: randomizer__.unwrap_or_default(),
                    value_blinding: value_blinding__.unwrap_or_default(),
                    proof_blinding_r: proof_blinding_r__.unwrap_or_default(),
                    proof_blinding_s: proof_blinding_s__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.SpendPlan", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SpendView {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spend_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.SpendView", len)?;
        if let Some(v) = self.spend_view.as_ref() {
            match v {
                spend_view::SpendView::Visible(v) => {
                    struct_ser.serialize_field("visible", v)?;
                }
                spend_view::SpendView::Opaque(v) => {
                    struct_ser.serialize_field("opaque", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SpendView {
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SpendView;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.SpendView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SpendView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Visible => {
                            if spend_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("visible"));
                            }
                            spend_view__ = map.next_value::<::std::option::Option<_>>()?.map(spend_view::SpendView::Visible)
;
                        }
                        GeneratedField::Opaque => {
                            if spend_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("opaque"));
                            }
                            spend_view__ = map.next_value::<::std::option::Option<_>>()?.map(spend_view::SpendView::Opaque)
;
                        }
                    }
                }
                Ok(SpendView {
                    spend_view: spend_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.SpendView", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for spend_view::Opaque {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spend.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.SpendView.Opaque", len)?;
        if let Some(v) = self.spend.as_ref() {
            struct_ser.serialize_field("spend", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for spend_view::Opaque {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = spend_view::Opaque;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.SpendView.Opaque")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<spend_view::Opaque, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = map.next_value()?;
                        }
                    }
                }
                Ok(spend_view::Opaque {
                    spend: spend__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.SpendView.Opaque", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for spend_view::Visible {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.spend.is_some() {
            len += 1;
        }
        if self.note.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.SpendView.Visible", len)?;
        if let Some(v) = self.spend.as_ref() {
            struct_ser.serialize_field("spend", v)?;
        }
        if let Some(v) = self.note.as_ref() {
            struct_ser.serialize_field("note", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for spend_view::Visible {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "spend",
            "note",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Spend,
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
                            "spend" => Ok(GeneratedField::Spend),
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
            type Value = spend_view::Visible;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.SpendView.Visible")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<spend_view::Visible, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut spend__ = None;
                let mut note__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Spend => {
                            if spend__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spend"));
                            }
                            spend__ = map.next_value()?;
                        }
                        GeneratedField::Note => {
                            if note__.is_some() {
                                return Err(serde::de::Error::duplicate_field("note"));
                            }
                            note__ = map.next_value()?;
                        }
                    }
                }
                Ok(spend_view::Visible {
                    spend: spend__,
                    note: note__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.SpendView.Visible", FIELDS, GeneratedVisitor)
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
        if !self.binding_sig.is_empty() {
            len += 1;
        }
        if self.anchor.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.Transaction", len)?;
        if let Some(v) = self.body.as_ref() {
            struct_ser.serialize_field("body", v)?;
        }
        if !self.binding_sig.is_empty() {
            struct_ser.serialize_field("bindingSig", pbjson::private::base64::encode(&self.binding_sig).as_str())?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.Transaction")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Transaction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body__ = None;
                let mut binding_sig__ = None;
                let mut anchor__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = map.next_value()?;
                        }
                        GeneratedField::BindingSig => {
                            if binding_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bindingSig"));
                            }
                            binding_sig__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map.next_value()?;
                        }
                    }
                }
                Ok(Transaction {
                    body: body__,
                    binding_sig: binding_sig__.unwrap_or_default(),
                    anchor: anchor__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.Transaction", FIELDS, GeneratedVisitor)
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
        if self.expiry_height != 0 {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        if !self.fmd_clues.is_empty() {
            len += 1;
        }
        if self.encrypted_memo.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.TransactionBody", len)?;
        if !self.actions.is_empty() {
            struct_ser.serialize_field("actions", &self.actions)?;
        }
        if self.expiry_height != 0 {
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if !self.fmd_clues.is_empty() {
            struct_ser.serialize_field("fmdClues", &self.fmd_clues)?;
        }
        if let Some(v) = self.encrypted_memo.as_ref() {
            struct_ser.serialize_field("encryptedMemo", pbjson::private::base64::encode(&v).as_str())?;
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
            "expiry_height",
            "expiryHeight",
            "chain_id",
            "chainId",
            "fee",
            "fmd_clues",
            "fmdClues",
            "encrypted_memo",
            "encryptedMemo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Actions,
            ExpiryHeight,
            ChainId,
            Fee,
            FmdClues,
            EncryptedMemo,
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
                            "expiryHeight" | "expiry_height" => Ok(GeneratedField::ExpiryHeight),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "fee" => Ok(GeneratedField::Fee),
                            "fmdClues" | "fmd_clues" => Ok(GeneratedField::FmdClues),
                            "encryptedMemo" | "encrypted_memo" => Ok(GeneratedField::EncryptedMemo),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.TransactionBody")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionBody, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut actions__ = None;
                let mut expiry_height__ = None;
                let mut chain_id__ = None;
                let mut fee__ = None;
                let mut fmd_clues__ = None;
                let mut encrypted_memo__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Actions => {
                            if actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actions"));
                            }
                            actions__ = Some(map.next_value()?);
                        }
                        GeneratedField::ExpiryHeight => {
                            if expiry_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryHeight"));
                            }
                            expiry_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::FmdClues => {
                            if fmd_clues__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdClues"));
                            }
                            fmd_clues__ = Some(map.next_value()?);
                        }
                        GeneratedField::EncryptedMemo => {
                            if encrypted_memo__.is_some() {
                                return Err(serde::de::Error::duplicate_field("encryptedMemo"));
                            }
                            encrypted_memo__ = 
                                map.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(TransactionBody {
                    actions: actions__.unwrap_or_default(),
                    expiry_height: expiry_height__.unwrap_or_default(),
                    chain_id: chain_id__.unwrap_or_default(),
                    fee: fee__,
                    fmd_clues: fmd_clues__.unwrap_or_default(),
                    encrypted_memo: encrypted_memo__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.TransactionBody", FIELDS, GeneratedVisitor)
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
        if self.expiry_height != 0 {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        if !self.fmd_clues.is_empty() {
            len += 1;
        }
        if self.memo_view.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.TransactionBodyView", len)?;
        if !self.action_views.is_empty() {
            struct_ser.serialize_field("actionViews", &self.action_views)?;
        }
        if self.expiry_height != 0 {
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if !self.fmd_clues.is_empty() {
            struct_ser.serialize_field("fmdClues", &self.fmd_clues)?;
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
            "expiry_height",
            "expiryHeight",
            "chain_id",
            "chainId",
            "fee",
            "fmd_clues",
            "fmdClues",
            "memo_view",
            "memoView",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ActionViews,
            ExpiryHeight,
            ChainId,
            Fee,
            FmdClues,
            MemoView,
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
                            "expiryHeight" | "expiry_height" => Ok(GeneratedField::ExpiryHeight),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "fee" => Ok(GeneratedField::Fee),
                            "fmdClues" | "fmd_clues" => Ok(GeneratedField::FmdClues),
                            "memoView" | "memo_view" => Ok(GeneratedField::MemoView),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.TransactionBodyView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionBodyView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut action_views__ = None;
                let mut expiry_height__ = None;
                let mut chain_id__ = None;
                let mut fee__ = None;
                let mut fmd_clues__ = None;
                let mut memo_view__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ActionViews => {
                            if action_views__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actionViews"));
                            }
                            action_views__ = Some(map.next_value()?);
                        }
                        GeneratedField::ExpiryHeight => {
                            if expiry_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryHeight"));
                            }
                            expiry_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::FmdClues => {
                            if fmd_clues__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fmdClues"));
                            }
                            fmd_clues__ = Some(map.next_value()?);
                        }
                        GeneratedField::MemoView => {
                            if memo_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memoView"));
                            }
                            memo_view__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionBodyView {
                    action_views: action_views__.unwrap_or_default(),
                    expiry_height: expiry_height__.unwrap_or_default(),
                    chain_id: chain_id__.unwrap_or_default(),
                    fee: fee__,
                    fmd_clues: fmd_clues__.unwrap_or_default(),
                    memo_view: memo_view__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.TransactionBodyView", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.TransactionPerspective", len)?;
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
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PayloadKeys,
            SpendNullifiers,
            AdviceNotes,
            AddressViews,
            Denoms,
            TransactionId,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.TransactionPerspective")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionPerspective, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut payload_keys__ = None;
                let mut spend_nullifiers__ = None;
                let mut advice_notes__ = None;
                let mut address_views__ = None;
                let mut denoms__ = None;
                let mut transaction_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PayloadKeys => {
                            if payload_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("payloadKeys"));
                            }
                            payload_keys__ = Some(map.next_value()?);
                        }
                        GeneratedField::SpendNullifiers => {
                            if spend_nullifiers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("spendNullifiers"));
                            }
                            spend_nullifiers__ = Some(map.next_value()?);
                        }
                        GeneratedField::AdviceNotes => {
                            if advice_notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("adviceNotes"));
                            }
                            advice_notes__ = Some(map.next_value()?);
                        }
                        GeneratedField::AddressViews => {
                            if address_views__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressViews"));
                            }
                            address_views__ = Some(map.next_value()?);
                        }
                        GeneratedField::Denoms => {
                            if denoms__.is_some() {
                                return Err(serde::de::Error::duplicate_field("denoms"));
                            }
                            denoms__ = Some(map.next_value()?);
                        }
                        GeneratedField::TransactionId => {
                            if transaction_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transactionId"));
                            }
                            transaction_id__ = map.next_value()?;
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
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.TransactionPerspective", FIELDS, GeneratedVisitor)
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
        if self.expiry_height != 0 {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        if self.fee.is_some() {
            len += 1;
        }
        if !self.clue_plans.is_empty() {
            len += 1;
        }
        if self.memo_plan.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.TransactionPlan", len)?;
        if !self.actions.is_empty() {
            struct_ser.serialize_field("actions", &self.actions)?;
        }
        if self.expiry_height != 0 {
            struct_ser.serialize_field("expiryHeight", ToString::to_string(&self.expiry_height).as_str())?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.fee.as_ref() {
            struct_ser.serialize_field("fee", v)?;
        }
        if !self.clue_plans.is_empty() {
            struct_ser.serialize_field("cluePlans", &self.clue_plans)?;
        }
        if let Some(v) = self.memo_plan.as_ref() {
            struct_ser.serialize_field("memoPlan", v)?;
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
            "expiry_height",
            "expiryHeight",
            "chain_id",
            "chainId",
            "fee",
            "clue_plans",
            "cluePlans",
            "memo_plan",
            "memoPlan",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Actions,
            ExpiryHeight,
            ChainId,
            Fee,
            CluePlans,
            MemoPlan,
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
                            "expiryHeight" | "expiry_height" => Ok(GeneratedField::ExpiryHeight),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "fee" => Ok(GeneratedField::Fee),
                            "cluePlans" | "clue_plans" => Ok(GeneratedField::CluePlans),
                            "memoPlan" | "memo_plan" => Ok(GeneratedField::MemoPlan),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.TransactionPlan")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionPlan, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut actions__ = None;
                let mut expiry_height__ = None;
                let mut chain_id__ = None;
                let mut fee__ = None;
                let mut clue_plans__ = None;
                let mut memo_plan__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Actions => {
                            if actions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("actions"));
                            }
                            actions__ = Some(map.next_value()?);
                        }
                        GeneratedField::ExpiryHeight => {
                            if expiry_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expiryHeight"));
                            }
                            expiry_height__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                        GeneratedField::Fee => {
                            if fee__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fee"));
                            }
                            fee__ = map.next_value()?;
                        }
                        GeneratedField::CluePlans => {
                            if clue_plans__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cluePlans"));
                            }
                            clue_plans__ = Some(map.next_value()?);
                        }
                        GeneratedField::MemoPlan => {
                            if memo_plan__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memoPlan"));
                            }
                            memo_plan__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionPlan {
                    actions: actions__.unwrap_or_default(),
                    expiry_height: expiry_height__.unwrap_or_default(),
                    chain_id: chain_id__.unwrap_or_default(),
                    fee: fee__,
                    clue_plans: clue_plans__.unwrap_or_default(),
                    memo_plan: memo_plan__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.TransactionPlan", FIELDS, GeneratedVisitor)
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
        if !self.binding_sig.is_empty() {
            len += 1;
        }
        if self.anchor.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.TransactionView", len)?;
        if let Some(v) = self.body_view.as_ref() {
            struct_ser.serialize_field("bodyView", v)?;
        }
        if !self.binding_sig.is_empty() {
            struct_ser.serialize_field("bindingSig", pbjson::private::base64::encode(&self.binding_sig).as_str())?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.TransactionView")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TransactionView, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut body_view__ = None;
                let mut binding_sig__ = None;
                let mut anchor__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::BodyView => {
                            if body_view__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bodyView"));
                            }
                            body_view__ = map.next_value()?;
                        }
                        GeneratedField::BindingSig => {
                            if binding_sig__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bindingSig"));
                            }
                            binding_sig__ = 
                                Some(map.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map.next_value()?;
                        }
                    }
                }
                Ok(TransactionView {
                    body_view: body_view__,
                    binding_sig: binding_sig__.unwrap_or_default(),
                    anchor: anchor__,
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.TransactionView", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("penumbra.core.transaction.v1alpha1.WitnessData", len)?;
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
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
                formatter.write_str("struct penumbra.core.transaction.v1alpha1.WitnessData")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<WitnessData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut anchor__ = None;
                let mut state_commitment_proofs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Anchor => {
                            if anchor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("anchor"));
                            }
                            anchor__ = map.next_value()?;
                        }
                        GeneratedField::StateCommitmentProofs => {
                            if state_commitment_proofs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stateCommitmentProofs"));
                            }
                            state_commitment_proofs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(WitnessData {
                    anchor: anchor__,
                    state_commitment_proofs: state_commitment_proofs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("penumbra.core.transaction.v1alpha1.WitnessData", FIELDS, GeneratedVisitor)
    }
}
