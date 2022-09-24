/// A transaction action that submits a swap to the dex.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Swap {
    /// Contains the Swap proof.
    #[prost(bytes="vec", tag="1")]
    #[serde(with = "crate::serializers::base64str")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// MockFlowCiphertext dropped until flow encryption/ABCI++ available
    /// // Encrypted amount of asset 1 of the trading pair.
    /// MockFlowCiphertext enc_amount_1 = 2;
    /// // Encrypted amount of asset 2 of the trading pair.
    /// MockFlowCiphertext enc_amount_2 = 3;
    /// Encapsulates the authorized fields of the Swap action, used in signing.
    #[prost(message, optional, tag="4")]
    pub body: ::core::option::Option<SwapBody>,
}
/// A transaction action that obtains assets previously confirmed
/// via a Swap transaction. Does not include a spend authorization
/// signature, as it is only capable of consuming the NFT from a
/// Swap transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaim {
    /// Contains the SwapClaim proof.
    #[prost(bytes="vec", tag="1")]
    #[serde(with = "crate::serializers::base64str")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// Encapsulates the authorized fields of the SwapClaim action, used in signing.
    #[prost(message, optional, tag="2")]
    pub body: ::core::option::Option<SwapClaimBody>,
}
/// Encapsulates the authorized fields of the SwapClaim action, used in signing.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimBody {
    /// The nullifier for the Swap NFT to be consumed.
    #[prost(message, optional, tag="1")]
    pub nullifier: ::core::option::Option<super::super::crypto::v1alpha1::Nullifier>,
    /// The fee allows `SwapClaim` without an additional `Spend`.
    #[prost(message, optional, tag="2")]
    pub fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// Note output for asset 1.
    #[prost(message, optional, tag="3")]
    pub output_1: ::core::option::Option<super::super::crypto::v1alpha1::NotePayload>,
    /// Note output for asset 2.
    #[prost(message, optional, tag="4")]
    pub output_2: ::core::option::Option<super::super::crypto::v1alpha1::NotePayload>,
    /// Input and output amounts, and asset IDs for the assets in the swap.
    #[prost(message, optional, tag="6")]
    pub output_data: ::core::option::Option<BatchSwapOutputData>,
    /// The epoch duration of the chain when the swap claim took place.
    #[prost(uint64, tag="7")]
    pub epoch_duration: u64,
}
/// For storing the list of claimed swaps between the dex and shielded pool components.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClaimedSwapList {
    #[prost(message, repeated, tag="1")]
    pub claims: ::prost::alloc::vec::Vec<ClaimedSwap>,
}
/// Represents a swap claimed in a particular transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClaimedSwap {
    #[prost(message, optional, tag="1")]
    pub claim: ::core::option::Option<SwapClaimBody>,
    #[prost(bytes="vec", tag="2")]
    pub txid: ::prost::alloc::vec::Vec<u8>,
}
/// The authorized data of a Swap transaction.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapBody {
    /// The trading pair to swap.
    #[prost(message, optional, tag="1")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// @exclude These will become commitments when flow encryption/ABCI++ are available
    /// The amount for asset 1.
    #[prost(message, optional, tag="2")]
    pub delta_1_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// The amount for asset 2.
    #[prost(message, optional, tag="3")]
    pub delta_2_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// @exclude // Commitment to the amount for asset 1 (delta 1).
    /// @exclude bytes delta_1_commitment = 2;
    /// @exclude // Commitment to the amount for asset 2 (delta 2).
    /// @exclude bytes delta_2_commitment = 3;
    /// A commitment to a prepaid fee for the future SwapClaim.
    #[prost(bytes="vec", tag="4")]
    pub fee_commitment: ::prost::alloc::vec::Vec<u8>,
    /// Swap NFT recording the user's contribution.
    #[prost(message, optional, tag="5")]
    pub swap_nft: ::core::option::Option<super::super::crypto::v1alpha1::NotePayload>,
    /// Encrypted version of the original `Swap`, symmetrically encrypted w/ viewing key.
    #[prost(bytes="vec", tag="6")]
    pub swap_ciphertext: ::prost::alloc::vec::Vec<u8>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPlaintext {
    /// The trading pair to swap.
    #[prost(message, optional, tag="1")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// Input amount of asset 1
    #[prost(message, optional, tag="2")]
    pub delta_1_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// Input amount of asset 2
    #[prost(message, optional, tag="3")]
    pub delta_2_i: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    /// Pre-paid fee to claim the swap
    #[prost(message, optional, tag="4")]
    pub claim_fee: ::core::option::Option<super::super::crypto::v1alpha1::Fee>,
    /// Address that will claim the swap outputs via SwapClaim.
    #[prost(message, optional, tag="5")]
    pub claim_address: ::core::option::Option<super::super::crypto::v1alpha1::Address>,
}
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(transparent)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MockFlowCiphertext {
    /// Represents this transaction's contribution to flow's value.
    #[prost(uint64, tag="1")]
    pub value: u64,
}
/// Holds two asset IDs. Ordering doesn't reflect trading direction, however
/// since the `AssetId` type is `Ord + PartialOrd`, there can be only one
/// `TradingPair` per asset pair.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TradingPair {
    /// The first asset of the pair.
    #[prost(message, optional, tag="1")]
    pub asset_1: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
    /// The second asset of the pair.
    #[prost(message, optional, tag="2")]
    pub asset_2: ::core::option::Option<super::super::crypto::v1alpha1::AssetId>,
}
/// Records the result of a batch swap on-chain.
///
/// Used as a public input to a swap claim proof, as it implies the effective
/// clearing price for the batch.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchSwapOutputData {
    /// The total amount of asset 1 that was input to the batch swap.
    #[prost(uint64, tag="1")]
    pub delta_1: u64,
    /// The total amount of asset 2 that was input to the batch swap.
    #[prost(uint64, tag="2")]
    pub delta_2: u64,
    /// The total amount of asset 1 that was output from the batch swap.
    #[prost(uint64, tag="3")]
    pub lambda_1: u64,
    /// The total amount of asset 2 that was output from the batch swap.
    #[prost(uint64, tag="4")]
    pub lambda_2: u64,
    /// Whether the swap succeeded or not.
    #[prost(bool, tag="5")]
    pub success: bool,
    /// The height for which the batch swap data is valid.
    #[prost(uint64, tag="6")]
    pub height: u64,
    /// The trading pair associated with the batch swap.
    #[prost(message, optional, tag="7")]
    pub trading_pair: ::core::option::Option<TradingPair>,
}
/// The data describing a trading function.
///
/// This implicitly treats the trading function as being between assets 1 and 2,
/// without specifying what those assets are, to avoid duplicating data (each
/// asset ID alone is twice the size of the trading function).
///
/// The trading function is `phi(R) = p*R_1 + q*R_2`.
/// This is used as a CFMM with constant `k` and fee `fee` (gamma).
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TradingFunction {
    /// NOTE: the use of floats here is a placeholder, so we can stub out the
    /// implementation and then decide what type of fixed-point, deterministic
    /// arithmetic should be used.
    #[prost(double, tag="2")]
    pub fee: f64,
    #[prost(double, tag="3")]
    pub k: f64,
    #[prost(double, tag="4")]
    pub p: f64,
    #[prost(double, tag="5")]
    pub q: f64,
}
/// The reserves of a position.
///
/// Like a position, this implicitly treats the trading function as being
/// between assets 1 and 2, without specifying what those assets are, to avoid
/// duplicating data (each asset ID alone is four times the size of the
/// reserves).
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Reserves {
    #[prost(message, optional, tag="1")]
    pub r1: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
    #[prost(message, optional, tag="2")]
    pub r2: ::core::option::Option<super::super::crypto::v1alpha1::Amount>,
}
/// Data identifying a position.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Position {
    #[prost(message, optional, tag="1")]
    pub pair: ::core::option::Option<TradingPair>,
    #[prost(message, optional, tag="2")]
    pub phi: ::core::option::Option<TradingFunction>,
    /// A random value used to disambiguate different positions with the exact same
    /// trading function.  The chain should reject newly created positions with the
    /// same nonce as an existing position.  This ensures that `PositionId`s will
    /// be unique, and allows us to track position ownership with a
    /// sequence of stateful NFTs based on the `PositionId`.
    #[prost(bytes="vec", tag="3")]
    #[serde(with = "crate::serializers::hexstr")]
    pub nonce: ::prost::alloc::vec::Vec<u8>,
}
/// A hash of a `Position`.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[serde(transparent)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionId {
    #[prost(bytes="vec", tag="1")]
    #[serde(with = "crate::serializers::bech32str::lp_id")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
/// The state of a position.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionState {
    #[prost(enumeration="position_state::PositionStateEnum", tag="1")]
    pub state: i32,
}
/// Nested message and enum types in `PositionState`.
pub mod position_state {
    #[derive(::serde::Deserialize, ::serde::Serialize)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum PositionStateEnum {
        /// The position has been opened, is active, has reserves and accumulated
        /// fees, and can be traded against.
        Opened = 0,
        /// The position has been closed, is inactive and can no longer be traded
        /// against, but still has reserves and accumulated fees.
        Closed = 1,
        /// The final reserves and accumulated fees have been withdrawn, leaving an
        /// empty, inactive position awaiting (possible) retroactive rewards.
        Withdrawn = 2,
        /// Any retroactive rewards have been claimed. The position is now an inert,
        /// historical artefact.
        Claimed = 3,
    }
    impl PositionStateEnum {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                PositionStateEnum::Opened => "OPENED",
                PositionStateEnum::Closed => "CLOSED",
                PositionStateEnum::Withdrawn => "WITHDRAWN",
                PositionStateEnum::Claimed => "CLAIMED",
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LpNft {
    #[prost(message, optional, tag="1")]
    pub position_id: ::core::option::Option<PositionId>,
    #[prost(message, optional, tag="2")]
    pub state: ::core::option::Option<PositionState>,
}
/// A transaction action that opens a new position.
///
/// This action's contribution to the transaction's value balance is to consume
/// the initial reserves and contribute an opened position NFT.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionOpen {
    /// Contains the data defining the position, sufficient to compute its `PositionId`.
    ///
    /// Positions are immutable, so the `PositionData` (and hence the `PositionId`)
    /// are unchanged over the entire lifetime of the position.
    #[prost(message, optional, tag="1")]
    pub position: ::core::option::Option<Position>,
    /// The initial reserves of the position.  Unlike the `PositionData`, the
    /// reserves evolve over time as trades are executed against the position.
    #[prost(message, optional, tag="2")]
    pub initial_reserves: ::core::option::Option<Reserves>,
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
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionClose {
    #[prost(message, optional, tag="1")]
    pub position_id: ::core::option::Option<PositionId>,
}
/// A transaction action that withdraws funds from a closed position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// closed position NFT and contribute a withdrawn position NFT, as well as all
/// of the funds that were in the position at the time of closing.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionWithdraw {
    #[prost(message, optional, tag="1")]
    pub position_id: ::core::option::Option<PositionId>,
    /// A transparent (zero blinding factor) commitment to the position's final reserves and fees.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    #[prost(message, optional, tag="2")]
    pub reserves_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
}
/// A transaction action that claims retroactive rewards for a historical
/// position.
///
/// This action's contribution to the transaction's value balance is to consume a
/// withdrawn position NFT and contribute its reward balance.
#[derive(::serde::Deserialize, ::serde::Serialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionRewardClaim {
    #[prost(message, optional, tag="1")]
    pub position_id: ::core::option::Option<PositionId>,
    /// A transparent (zero blinding factor) commitment to the position's accumulated rewards.
    ///
    /// The chain will check this commitment by recomputing it with the on-chain state.
    #[prost(message, optional, tag="2")]
    pub rewards_commitment: ::core::option::Option<super::super::crypto::v1alpha1::BalanceCommitment>,
}
