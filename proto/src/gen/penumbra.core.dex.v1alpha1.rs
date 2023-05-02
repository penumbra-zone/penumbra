/// A transaction action that submits a swap to the dex.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Swap {
    /// Contains the Swap proof.
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<super::super::crypto::v1alpha1::ZkSwapProof>,
    /// MockFlowCiphertext dropped until flow encryption/ABCI++ available
    /// // Encrypted amount of asset 1 of the trading pair.
    /// MockFlowCiphertext enc_amount_1 = 2;
    /// // Encrypted amount of asset 2 of the trading pair.
    /// MockFlowCiphertext enc_amount_2 = 3;
    /// Encapsulates the authorized fields of the Swap action, used in signing.
    #[prost(message, optional, tag = "4")]
    pub body: ::core::option::Option<SwapBody>,
}
/// A transaction action that obtains assets previously confirmed
/// via a Swap transaction. Does not include a spend authorization
/// signature, as it is only capable of consuming the NFT from a
/// Swap transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaim {
    /// Contains the SwapClaim proof.
    #[prost(bytes = "vec", tag = "1")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// Encapsulates the authorized fields of the SwapClaim action, used in signing.
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<SwapClaimBody>,
    /// The epoch duration of the chain when the swap claim took place.
    #[prost(uint64, tag = "7")]
    pub epoch_duration: u64,
}
/// Encapsulates the authorized fields of the SwapClaim action, used in signing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimBody {
    /// The nullifier for the Swap commitment to be consumed.
    #[prost(message, optional, tag = "1")]
    pub nullifier: ::core::option::Option<super::super::crypto::v1alpha1::Nullifier>,
    /// The fee allows `SwapClaim` without an additional `Spend`.
    #[prost(message, optional, tag = "2")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// Note output for asset 1.
    #[prost(message, optional, tag = "3")]
    pub output_1_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::StateCommitment,
    >,
    /// Note output for asset 2.
    #[prost(message, optional, tag = "4")]
    pub output_2_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::StateCommitment,
    >,
    /// Input and output amounts, and asset IDs for the assets in the swap.
    #[prost(message, optional, tag = "6")]
    pub output_data: ::core::option::Option<BatchSwapOutputData>,
}
/// The authorized data of a Swap transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapBody {
    /// The trading pair to swap.
    #[prost(message, optional, tag = "1")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// The amount for asset 1.
    #[prost(message, optional, tag = "2")]
    pub delta_1_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The amount for asset 2.
    #[prost(message, optional, tag = "3")]
    pub delta_2_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// A commitment to a prepaid fee for the future SwapClaim.
    /// This is recorded separately from delta_j_i because it's shielded;
    /// in the future we'll want separate commitments to each delta_j_i
    /// anyways in order to prove consistency with flow encryption.
    #[prost(message, optional, tag = "4")]
    pub fee_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::BalanceCommitment,
    >,
    /// The swap commitment and encryption of the swap data.
    #[prost(message, optional, tag = "5")]
    pub payload: ::core::option::Option<SwapPayload>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPayload {
    #[prost(message, optional, tag = "1")]
    pub commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::StateCommitment,
    >,
    #[prost(bytes = "vec", tag = "2")]
    pub encrypted_swap: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPlaintext {
    /// The trading pair to swap.
    #[prost(message, optional, tag = "1")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// Input amount of asset 1
    #[prost(message, optional, tag = "2")]
    pub delta_1_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// Input amount of asset 2
    #[prost(message, optional, tag = "3")]
    pub delta_2_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// Pre-paid fee to claim the swap
    #[prost(message, optional, tag = "4")]
    pub claim_fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// Address that will claim the swap outputs via SwapClaim.
    #[prost(message, optional, tag = "5")]
    pub claim_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
    /// Swap rseed (blinding factors are derived from this)
    #[prost(bytes = "vec", tag = "6")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MockFlowCiphertext {
    /// Represents this transaction's contribution to flow's value.
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPlan {
    /// The plaintext version of the swap to be performed.
    #[prost(message, optional, tag = "1")]
    pub swap_plaintext: ::core::option::Option<SwapPlaintext>,
    /// The blinding factor for the fee commitment. The fee in the SwapPlan is private to prevent linkability with the SwapClaim.
    #[prost(bytes = "vec", tag = "2")]
    pub fee_blinding: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimPlan {
    /// The plaintext version of the swap to be performed.
    #[prost(message, optional, tag = "1")]
    pub swap_plaintext: ::core::option::Option<SwapPlaintext>,
    /// The position of the swap commitment.
    #[prost(uint64, tag = "2")]
    pub position: u64,
    /// Input and output amounts for the Swap.
    #[prost(message, optional, tag = "3")]
    pub output_data: ::core::option::Option<BatchSwapOutputData>,
    /// The epoch duration, used in proving.
    #[prost(uint64, tag = "4")]
    pub epoch_duration: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapView {
    #[prost(oneof = "swap_view::SwapView", tags = "1, 2")]
    pub swap_view: ::core::option::Option<swap_view::SwapView>,
}
/// Nested message and enum types in `SwapView`.
pub mod swap_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub swap: ::core::option::Option<super::Swap>,
        #[prost(message, optional, tag = "3")]
        pub swap_plaintext: ::core::option::Option<super::SwapPlaintext>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub swap: ::core::option::Option<super::Swap>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SwapView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimView {
    #[prost(oneof = "swap_claim_view::SwapClaimView", tags = "1, 2")]
    pub swap_claim_view: ::core::option::Option<swap_claim_view::SwapClaimView>,
}
/// Nested message and enum types in `SwapClaimView`.
pub mod swap_claim_view {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Visible {
        #[prost(message, optional, tag = "1")]
        pub swap_claim: ::core::option::Option<super::SwapClaim>,
        #[prost(message, optional, tag = "2")]
        pub output_1: ::core::option::Option<
            super::super::super::crypto::v1alpha1::NoteView,
        >,
        #[prost(message, optional, tag = "3")]
        pub output_2: ::core::option::Option<
            super::super::super::crypto::v1alpha1::NoteView,
        >,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub swap_claim: ::core::option::Option<super::SwapClaim>,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SwapClaimView {
        #[prost(message, tag = "1")]
        Visible(Visible),
        #[prost(message, tag = "2")]
        Opaque(Opaque),
    }
}
/// Holds two asset IDs. Ordering doesn't reflect trading direction. Instead, we
/// require `asset_1 < asset_2` as field elements, to ensure a canonical
/// representation of an unordered pair.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TradingPair {
    /// The first asset of the pair.
    #[prost(message, optional, tag = "1")]
    pub asset_1: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
    /// The second asset of the pair.
    #[prost(message, optional, tag = "2")]
    pub asset_2: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
}
/// Encodes a trading pair starting from asset `start`
/// and ending on asset `end`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DirectedTradingPair {
    /// The start asset of the pair.
    #[prost(message, optional, tag = "1")]
    pub start: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
    /// The end asset of the pair.
    #[prost(message, optional, tag = "2")]
    pub end: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
}
/// Records the result of a batch swap on-chain.
///
/// Used as a public input to a swap claim proof, as it implies the effective
/// clearing price for the batch.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchSwapOutputData {
    /// The total amount of asset 1 that was input to the batch swap.
    #[prost(message, optional, tag = "1")]
    pub delta_1: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The total amount of asset 2 that was input to the batch swap.
    #[prost(message, optional, tag = "2")]
    pub delta_2: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The total amount of asset 1 that was output from the batch swap for 1=>2 trades.
    #[prost(message, optional, tag = "3")]
    pub lambda_1_1: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The total amount of asset 2 that was output from the batch swap for 1=>2 trades.
    #[prost(message, optional, tag = "4")]
    pub lambda_2_1: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The total amount of asset 1 that was output from the batch swap for 2=>1 trades.
    #[prost(message, optional, tag = "5")]
    pub lambda_1_2: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The total amount of asset 2 that was output from the batch swap for 2=>1 trades.
    #[prost(message, optional, tag = "6")]
    pub lambda_2_2: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The height for which the batch swap data is valid.
    #[prost(uint64, tag = "7")]
    pub height: u64,
    /// The trading pair associated with the batch swap.
    #[prost(message, optional, tag = "8")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// The starting block height of the epoch for which the batch swap data is valid.
    #[prost(uint64, tag = "9")]
    pub epoch_height: u64,
}
/// The trading function for a specific pair.
/// For a pair (asset_1, asset_2), a trading function is defined by:
/// `phi(R) = p*R_1 + q*R_2` and `gamma = 1 - fee`.
/// The trading function is frequently referred to as "phi".
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TradingFunction {
    #[prost(message, optional, tag = "1")]
    pub component: ::core::option::Option<BareTradingFunction>,
    #[prost(message, optional, tag = "2")]
    pub pair: ::core::option::Option<TradingPair>,
}
/// The minimum amount of data describing a trading function.
///
/// This implicitly treats the trading function as being between assets 1 and 2,
/// without specifying what those assets are, to avoid duplicating data (each
/// asset ID alone is twice the size of the trading function).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BareTradingFunction {
    #[prost(uint32, tag = "1")]
    pub fee: u32,
    /// This is not actually an amount, it's an integer the same width as an amount
    #[prost(message, optional, tag = "2")]
    pub p: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// This is not actually an amount, it's an integer the same width as an amount
    #[prost(message, optional, tag = "3")]
    pub q: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
/// The reserves of a position.
///
/// Like a position, this implicitly treats the trading function as being
/// between assets 1 and 2, without specifying what those assets are, to avoid
/// duplicating data (each asset ID alone is four times the size of the
/// reserves).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reserves {
    #[prost(message, optional, tag = "1")]
    pub r1: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    #[prost(message, optional, tag = "2")]
    pub r2: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
/// Data identifying a position.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Position {
    #[prost(message, optional, tag = "1")]
    pub phi: ::core::option::Option<TradingFunction>,
    /// A random value used to disambiguate different positions with the exact same
    /// trading function.  The chain should reject newly created positions with the
    /// same nonce as an existing position.  This ensures that `PositionId`s will
    /// be unique, and allows us to track position ownership with a
    /// sequence of stateful NFTs based on the `PositionId`.
    #[prost(bytes = "vec", tag = "2")]
    pub nonce: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub state: ::core::option::Option<PositionState>,
    #[prost(message, optional, tag = "4")]
    pub reserves: ::core::option::Option<Reserves>,
}
/// A hash of a `Position`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionId {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// The state of a position.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionState {
    #[prost(enumeration = "position_state::PositionStateEnum", tag = "1")]
    pub state: i32,
}
/// Nested message and enum types in `PositionState`.
pub mod position_state {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum PositionStateEnum {
        Unspecified = 0,
        /// The position has been opened, is active, has reserves and accumulated
        /// fees, and can be traded against.
        Opened = 1,
        /// The position has been closed, is inactive and can no longer be traded
        /// against, but still has reserves and accumulated fees.
        Closed = 2,
        /// The final reserves and accumulated fees have been withdrawn, leaving an
        /// empty, inactive position awaiting (possible) retroactive rewards.
        Withdrawn = 3,
        /// Any retroactive rewards have been claimed. The position is now an inert,
        /// historical artefact.
        Claimed = 4,
    }
    impl PositionStateEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                PositionStateEnum::Unspecified => "POSITION_STATE_ENUM_UNSPECIFIED",
                PositionStateEnum::Opened => "POSITION_STATE_ENUM_OPENED",
                PositionStateEnum::Closed => "POSITION_STATE_ENUM_CLOSED",
                PositionStateEnum::Withdrawn => "POSITION_STATE_ENUM_WITHDRAWN",
                PositionStateEnum::Claimed => "POSITION_STATE_ENUM_CLAIMED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "POSITION_STATE_ENUM_UNSPECIFIED" => Some(Self::Unspecified),
                "POSITION_STATE_ENUM_OPENED" => Some(Self::Opened),
                "POSITION_STATE_ENUM_CLOSED" => Some(Self::Closed),
                "POSITION_STATE_ENUM_WITHDRAWN" => Some(Self::Withdrawn),
                "POSITION_STATE_ENUM_CLAIMED" => Some(Self::Claimed),
                _ => None,
            }
        }
    }
}
/// An LPNFT tracking both ownership and state of a position.
///
/// Tracking the state as part of the LPNFT means that all LP-related actions can
/// be authorized by spending funds: a state transition (e.g., closing a
/// position) is modeled as spending an "open position LPNFT" and minting a
/// "closed position LPNFT" for the same (globally unique) position ID.
///
/// This means that the LP mechanics can be agnostic to the mechanism used to
/// record custody and spend authorization.  For instance, they can be recorded
/// in the shielded pool, where custody is based on off-chain keys, or they could
/// be recorded in a programmatic on-chain account (in the future, e.g., to
/// support interchain accounts).  This also means that LP-related actions don't
/// require any cryptographic implementation (proofs, signatures, etc), other
/// than hooking into the value commitment mechanism used for transaction
/// balances.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LpNft {
    #[prost(message, optional, tag = "1")]
    pub position_id: ::core::option::Option<PositionId>,
    #[prost(message, optional, tag = "2")]
    pub state: ::core::option::Option<PositionState>,
}
/// A transaction action that opens a new position.
///
/// This action's contribution to the transaction's value balance is to consume
/// the initial reserves and contribute an opened position NFT.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionOpen {
    /// Contains the data defining the position, sufficient to compute its `PositionId`.
    ///
    /// Positions are immutable, so the `PositionData` (and hence the `PositionId`)
    /// are unchanged over the entire lifetime of the position.
    #[prost(message, optional, tag = "1")]
    pub position: ::core::option::Option<Position>,
}
/// A transaction action that closes a position.
///
/// This action's contribution to the transaction's value balance is to consume
/// an opened position NFT and contribute a closed position NFT.
///
/// Closing a position does not immediately withdraw funds, because Penumbra
/// transactions (like any ZK transaction model) are early-binding: the prover
/// must know the state transition they prove knowledge of, and they cannot know
/// the final reserves with certainty until after the position has been deactivated.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionClose {
    #[prost(message, optional, tag = "1")]
    pub position_id: ::core::option::Option<PositionId>,
}
/// A transaction action that withdraws funds from a closed position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// closed position NFT and contribute a withdrawn position NFT, as well as all
/// of the funds that were in the position at the time of closing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionWithdraw {
    #[prost(message, optional, tag = "1")]
    pub position_id: ::core::option::Option<PositionId>,
    /// A transparent (zero blinding factor) commitment to the position's final reserves and fees.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    #[prost(message, optional, tag = "2")]
    pub reserves_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::BalanceCommitment,
    >,
}
/// A transaction action that claims retroactive rewards for a historical
/// position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// withdrawn position NFT and contribute its reward balance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionRewardClaim {
    #[prost(message, optional, tag = "1")]
    pub position_id: ::core::option::Option<PositionId>,
    /// A transparent (zero blinding factor) commitment to the position's accumulated rewards.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    #[prost(message, optional, tag = "2")]
    pub rewards_commitment: ::core::option::Option<
        super::super::crypto::v1alpha1::BalanceCommitment,
    >,
}
/// Contains a path for a trade, including the trading pair (with direction), the trading
/// function defining their relationship, and the route taken between the two assets.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Path {
    #[prost(message, optional, tag = "1")]
    pub pair: ::core::option::Option<DirectedTradingPair>,
    #[prost(message, repeated, tag = "2")]
    pub route: ::prost::alloc::vec::Vec<super::super::crypto::v1alpha1::AssetId>,
    #[prost(message, optional, tag = "3")]
    pub phi: ::core::option::Option<BareTradingFunction>,
}
/// Contains the entire execution of a particular swap.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapExecution {
    #[prost(message, repeated, tag = "1")]
    pub traces: ::prost::alloc::vec::Vec<swap_execution::Trace>,
}
/// Nested message and enum types in `SwapExecution`.
pub mod swap_execution {
    /// Contains all individual steps consisting of a trade trace.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Trace {
        /// Each step in the trade trace.
        #[prost(message, repeated, tag = "1")]
        pub value: ::prost::alloc::vec::Vec<
            super::super::super::crypto::v1alpha1::Value,
        >,
    }
}
/// Contains private and public data for withdrawing funds from a closed position.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionWithdrawPlan {
    #[prost(message, optional, tag = "1")]
    pub reserves: ::core::option::Option<Reserves>,
    #[prost(message, optional, tag = "2")]
    pub position_id: ::core::option::Option<PositionId>,
    #[prost(message, optional, tag = "3")]
    pub pair: ::core::option::Option<TradingPair>,
}
/// Contains private and public data for claiming rewards from a position.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionRewardClaimPlan {
    #[prost(message, optional, tag = "1")]
    pub reserves: ::core::option::Option<Reserves>,
}
