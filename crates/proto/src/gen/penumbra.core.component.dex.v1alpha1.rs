/// A Penumbra ZK swap proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkSwapProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkSwapProof {
    const NAME: &'static str = "ZKSwapProof";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// A Penumbra ZK swap claim proof.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ZkSwapClaimProof {
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ZkSwapClaimProof {
    const NAME: &'static str = "ZKSwapClaimProof";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction action that submits a swap to the dex.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Swap {
    /// Contains the Swap proof.
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<ZkSwapProof>,
    /// Encapsulates the authorized fields of the Swap action, used in signing.
    #[prost(message, optional, tag = "4")]
    pub body: ::core::option::Option<SwapBody>,
}
impl ::prost::Name for Swap {
    const NAME: &'static str = "Swap";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// A transaction action that obtains assets previously confirmed
/// via a Swap transaction. Does not include a spend authorization
/// signature, as it is only capable of consuming the NFT from a
/// Swap transaction.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaim {
    /// Contains the SwapClaim proof.
    #[prost(message, optional, tag = "1")]
    pub proof: ::core::option::Option<ZkSwapClaimProof>,
    /// Encapsulates the authorized fields of the SwapClaim action, used in signing.
    #[prost(message, optional, tag = "2")]
    pub body: ::core::option::Option<SwapClaimBody>,
    /// The epoch duration of the chain when the swap claim took place.
    #[prost(uint64, tag = "7")]
    pub epoch_duration: u64,
}
impl ::prost::Name for SwapClaim {
    const NAME: &'static str = "SwapClaim";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// Encapsulates the authorized fields of the SwapClaim action, used in signing.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapClaimBody {
    /// The nullifier for the Swap commitment to be consumed.
    #[prost(message, optional, tag = "1")]
    pub nullifier: ::core::option::Option<super::super::sct::v1alpha1::Nullifier>,
    /// The fee allows `SwapClaim` without an additional `Spend`.
    #[prost(message, optional, tag = "2")]
    pub fee: ::core::option::Option<super::super::fee::v1alpha1::Fee>,
    /// Note output for asset 1.
    #[prost(message, optional, tag = "3")]
    pub output_1_commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// Note output for asset 2.
    #[prost(message, optional, tag = "4")]
    pub output_2_commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    /// Input and output amounts, and asset IDs for the assets in the swap.
    #[prost(message, optional, tag = "6")]
    pub output_data: ::core::option::Option<BatchSwapOutputData>,
}
impl ::prost::Name for SwapClaimBody {
    const NAME: &'static str = "SwapClaimBody";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    pub delta_1_i: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The amount for asset 2.
    #[prost(message, optional, tag = "3")]
    pub delta_2_i: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// A commitment to a prepaid fee for the future SwapClaim.
    /// This is recorded separately from delta_j_i because it's shielded;
    /// in the future we'll want separate commitments to each delta_j_i
    /// anyways in order to prove consistency with flow encryption.
    #[prost(message, optional, tag = "4")]
    pub fee_commitment: ::core::option::Option<
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
    /// The swap commitment and encryption of the swap data.
    #[prost(message, optional, tag = "5")]
    pub payload: ::core::option::Option<SwapPayload>,
}
impl ::prost::Name for SwapBody {
    const NAME: &'static str = "SwapBody";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPayload {
    #[prost(message, optional, tag = "1")]
    pub commitment: ::core::option::Option<
        super::super::super::super::crypto::tct::v1alpha1::StateCommitment,
    >,
    #[prost(bytes = "vec", tag = "2")]
    pub encrypted_swap: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SwapPayload {
    const NAME: &'static str = "SwapPayload";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapPlaintext {
    /// The trading pair to swap.
    #[prost(message, optional, tag = "1")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// Input amount of asset 1
    #[prost(message, optional, tag = "2")]
    pub delta_1_i: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// Input amount of asset 2
    #[prost(message, optional, tag = "3")]
    pub delta_2_i: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// Pre-paid fee to claim the swap
    #[prost(message, optional, tag = "4")]
    pub claim_fee: ::core::option::Option<super::super::fee::v1alpha1::Fee>,
    /// Address that will claim the swap outputs via SwapClaim.
    #[prost(message, optional, tag = "5")]
    pub claim_address: ::core::option::Option<
        super::super::super::keys::v1alpha1::Address,
    >,
    /// Swap rseed (blinding factors are derived from this)
    #[prost(bytes = "vec", tag = "6")]
    pub rseed: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SwapPlaintext {
    const NAME: &'static str = "SwapPlaintext";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    /// The first blinding factor to use for the ZK swap proof.
    #[prost(bytes = "vec", tag = "3")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK swap proof.
    #[prost(bytes = "vec", tag = "4")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SwapPlan {
    const NAME: &'static str = "SwapPlan";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    /// The first blinding factor to use for the ZK swap claim proof.
    #[prost(bytes = "vec", tag = "5")]
    pub proof_blinding_r: ::prost::alloc::vec::Vec<u8>,
    /// The second blinding factor to use for the ZK swap claim proof.
    #[prost(bytes = "vec", tag = "6")]
    pub proof_blinding_s: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for SwapClaimPlan {
    const NAME: &'static str = "SwapClaimPlan";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.dex.v1alpha1.SwapView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub swap: ::core::option::Option<super::Swap>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.dex.v1alpha1.SwapView.{}", Self::NAME
            )
        }
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
impl ::prost::Name for SwapView {
    const NAME: &'static str = "SwapView";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
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
            super::super::super::shielded_pool::v1alpha1::NoteView,
        >,
        #[prost(message, optional, tag = "3")]
        pub output_2: ::core::option::Option<
            super::super::super::shielded_pool::v1alpha1::NoteView,
        >,
    }
    impl ::prost::Name for Visible {
        const NAME: &'static str = "Visible";
        const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.dex.v1alpha1.SwapClaimView.{}", Self::NAME
            )
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Opaque {
        #[prost(message, optional, tag = "1")]
        pub swap_claim: ::core::option::Option<super::SwapClaim>,
    }
    impl ::prost::Name for Opaque {
        const NAME: &'static str = "Opaque";
        const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.dex.v1alpha1.SwapClaimView.{}", Self::NAME
            )
        }
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
impl ::prost::Name for SwapClaimView {
    const NAME: &'static str = "SwapClaimView";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
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
    pub asset_1: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
    /// The second asset of the pair.
    #[prost(message, optional, tag = "2")]
    pub asset_2: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
}
impl ::prost::Name for TradingPair {
    const NAME: &'static str = "TradingPair";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// Encodes a trading pair starting from asset `start`
/// and ending on asset `end`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DirectedTradingPair {
    /// The start asset of the pair.
    #[prost(message, optional, tag = "1")]
    pub start: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
    /// The end asset of the pair.
    #[prost(message, optional, tag = "2")]
    pub end: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
}
impl ::prost::Name for DirectedTradingPair {
    const NAME: &'static str = "DirectedTradingPair";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    pub delta_1: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The total amount of asset 2 that was input to the batch swap.
    #[prost(message, optional, tag = "2")]
    pub delta_2: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The total amount of asset 1 that was output from the batch swap for 2=>1 trades.
    #[prost(message, optional, tag = "3")]
    pub lambda_1: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The total amount of asset 2 that was output from the batch swap for 1=>2 trades.
    #[prost(message, optional, tag = "4")]
    pub lambda_2: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The total amount of asset 1 that was returned unfilled from the batch swap for 1=>2 trades.
    #[prost(message, optional, tag = "5")]
    pub unfilled_1: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The total amount of asset 2 that was returned unfilled from the batch swap for 2=>1 trades.
    #[prost(message, optional, tag = "6")]
    pub unfilled_2: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// The height for which the batch swap data is valid.
    #[prost(uint64, tag = "7")]
    pub height: u64,
    /// The trading pair associated with the batch swap.
    #[prost(message, optional, tag = "8")]
    pub trading_pair: ::core::option::Option<TradingPair>,
    /// The starting block height of the epoch for which the batch swap data is valid.
    #[prost(uint64, tag = "9")]
    pub epoch_starting_height: u64,
}
impl ::prost::Name for BatchSwapOutputData {
    const NAME: &'static str = "BatchSwapOutputData";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
impl ::prost::Name for TradingFunction {
    const NAME: &'static str = "TradingFunction";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    pub p: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    /// This is not actually an amount, it's an integer the same width as an amount
    #[prost(message, optional, tag = "3")]
    pub q: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
}
impl ::prost::Name for BareTradingFunction {
    const NAME: &'static str = "BareTradingFunction";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    pub r1: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
    #[prost(message, optional, tag = "2")]
    pub r2: ::core::option::Option<super::super::super::num::v1alpha1::Amount>,
}
impl ::prost::Name for Reserves {
    const NAME: &'static str = "Reserves";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
    /// / If set to true, the position is a limit-order and will be closed
    /// / immediately after being filled.
    #[prost(bool, tag = "5")]
    pub close_on_fill: bool,
}
impl ::prost::Name for Position {
    const NAME: &'static str = "Position";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// A hash of a `Position`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionId {
    /// The bytes of the position ID.
    #[prost(bytes = "vec", tag = "1")]
    pub inner: ::prost::alloc::vec::Vec<u8>,
    /// Alternatively, a Bech32m-encoded string representation of the `inner`
    /// bytes.
    ///
    /// NOTE: implementations are not required to support parsing this field.
    /// Implementations should prefer to encode the bytes in all messages they
    /// produce. Implementations must not accept messages with both `inner` and
    /// `alt_bech32m` set.
    #[prost(string, tag = "2")]
    pub alt_bech32m: ::prost::alloc::string::String,
}
impl ::prost::Name for PositionId {
    const NAME: &'static str = "PositionId";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
impl ::prost::Name for PositionState {
    const NAME: &'static str = "PositionState";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
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
impl ::prost::Name for LpNft {
    const NAME: &'static str = "LpNft";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
impl ::prost::Name for PositionOpen {
    const NAME: &'static str = "PositionOpen";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
impl ::prost::Name for PositionClose {
    const NAME: &'static str = "PositionClose";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
}
impl ::prost::Name for PositionWithdraw {
    const NAME: &'static str = "PositionWithdraw";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
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
        super::super::super::asset::v1alpha1::BalanceCommitment,
    >,
}
impl ::prost::Name for PositionRewardClaim {
    const NAME: &'static str = "PositionRewardClaim";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// Contains the entire execution of a particular swap.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapExecution {
    #[prost(message, repeated, tag = "1")]
    pub traces: ::prost::alloc::vec::Vec<swap_execution::Trace>,
    /// The total input amount for this execution.
    #[prost(message, optional, tag = "2")]
    pub input: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    /// The total output amount for this execution.
    #[prost(message, optional, tag = "3")]
    pub output: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
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
            super::super::super::super::asset::v1alpha1::Value,
        >,
    }
    impl ::prost::Name for Trace {
        const NAME: &'static str = "Trace";
        const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.dex.v1alpha1.SwapExecution.{}", Self::NAME
            )
        }
    }
}
impl ::prost::Name for SwapExecution {
    const NAME: &'static str = "SwapExecution";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
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
impl ::prost::Name for PositionWithdrawPlan {
    const NAME: &'static str = "PositionWithdrawPlan";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// Contains private and public data for claiming rewards from a position.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PositionRewardClaimPlan {
    #[prost(message, optional, tag = "1")]
    pub reserves: ::core::option::Option<Reserves>,
}
impl ::prost::Name for PositionRewardClaimPlan {
    const NAME: &'static str = "PositionRewardClaimPlan";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// Requests batch swap data associated with a given height and trading pair from the view service.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchSwapOutputDataRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(message, optional, tag = "3")]
    pub trading_pair: ::core::option::Option<TradingPair>,
}
impl ::prost::Name for BatchSwapOutputDataRequest {
    const NAME: &'static str = "BatchSwapOutputDataRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatchSwapOutputDataResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<BatchSwapOutputData>,
}
impl ::prost::Name for BatchSwapOutputDataResponse {
    const NAME: &'static str = "BatchSwapOutputDataResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapExecutionRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(message, optional, tag = "3")]
    pub trading_pair: ::core::option::Option<DirectedTradingPair>,
}
impl ::prost::Name for SwapExecutionRequest {
    const NAME: &'static str = "SwapExecutionRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapExecutionResponse {
    #[prost(message, optional, tag = "1")]
    pub swap_execution: ::core::option::Option<SwapExecution>,
}
impl ::prost::Name for SwapExecutionResponse {
    const NAME: &'static str = "SwapExecutionResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArbExecutionRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub height: u64,
}
impl ::prost::Name for ArbExecutionRequest {
    const NAME: &'static str = "ArbExecutionRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArbExecutionResponse {
    #[prost(message, optional, tag = "1")]
    pub swap_execution: ::core::option::Option<SwapExecution>,
    #[prost(uint64, tag = "2")]
    pub height: u64,
}
impl ::prost::Name for ArbExecutionResponse {
    const NAME: &'static str = "ArbExecutionResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapExecutionsRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// If present, only return swap executions occurring after the given height.
    #[prost(uint64, tag = "2")]
    pub start_height: u64,
    /// If present, only return swap executions occurring before the given height.
    #[prost(uint64, tag = "3")]
    pub end_height: u64,
    /// If present, filter swap executions by the given trading pair.
    #[prost(message, optional, tag = "4")]
    pub trading_pair: ::core::option::Option<DirectedTradingPair>,
}
impl ::prost::Name for SwapExecutionsRequest {
    const NAME: &'static str = "SwapExecutionsRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SwapExecutionsResponse {
    #[prost(message, optional, tag = "1")]
    pub swap_execution: ::core::option::Option<SwapExecution>,
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(message, optional, tag = "3")]
    pub trading_pair: ::core::option::Option<DirectedTradingPair>,
}
impl ::prost::Name for SwapExecutionsResponse {
    const NAME: &'static str = "SwapExecutionsResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArbExecutionsRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// If present, only return arb executions occurring after the given height.
    #[prost(uint64, tag = "2")]
    pub start_height: u64,
    /// If present, only return arb executions occurring before the given height.
    #[prost(uint64, tag = "3")]
    pub end_height: u64,
}
impl ::prost::Name for ArbExecutionsRequest {
    const NAME: &'static str = "ArbExecutionsRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArbExecutionsResponse {
    #[prost(message, optional, tag = "1")]
    pub swap_execution: ::core::option::Option<SwapExecution>,
    #[prost(uint64, tag = "2")]
    pub height: u64,
}
impl ::prost::Name for ArbExecutionsResponse {
    const NAME: &'static str = "ArbExecutionsResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionsRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// If true, include closed and withdrawn positions.
    #[prost(bool, tag = "4")]
    pub include_closed: bool,
}
impl ::prost::Name for LiquidityPositionsRequest {
    const NAME: &'static str = "LiquidityPositionsRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionsResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<Position>,
}
impl ::prost::Name for LiquidityPositionsResponse {
    const NAME: &'static str = "LiquidityPositionsResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionByIdRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub position_id: ::core::option::Option<PositionId>,
}
impl ::prost::Name for LiquidityPositionByIdRequest {
    const NAME: &'static str = "LiquidityPositionByIdRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionByIdResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<Position>,
}
impl ::prost::Name for LiquidityPositionByIdResponse {
    const NAME: &'static str = "LiquidityPositionByIdResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionsByIdRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub position_id: ::prost::alloc::vec::Vec<PositionId>,
}
impl ::prost::Name for LiquidityPositionsByIdRequest {
    const NAME: &'static str = "LiquidityPositionsByIdRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionsByIdResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<Position>,
}
impl ::prost::Name for LiquidityPositionsByIdResponse {
    const NAME: &'static str = "LiquidityPositionsByIdResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionsByPriceRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    /// The directed trading pair to request positions for
    #[prost(message, optional, tag = "2")]
    pub trading_pair: ::core::option::Option<DirectedTradingPair>,
    /// The maximum number of positions to return.
    #[prost(uint64, tag = "5")]
    pub limit: u64,
}
impl ::prost::Name for LiquidityPositionsByPriceRequest {
    const NAME: &'static str = "LiquidityPositionsByPriceRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LiquidityPositionsByPriceResponse {
    #[prost(message, optional, tag = "1")]
    pub data: ::core::option::Option<Position>,
}
impl ::prost::Name for LiquidityPositionsByPriceResponse {
    const NAME: &'static str = "LiquidityPositionsByPriceResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpreadRequest {
    /// The expected chain id (empty string if no expectation).
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub trading_pair: ::core::option::Option<TradingPair>,
}
impl ::prost::Name for SpreadRequest {
    const NAME: &'static str = "SpreadRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpreadResponse {
    /// The best position when trading 1 => 2.
    #[prost(message, optional, tag = "1")]
    pub best_1_to_2_position: ::core::option::Option<Position>,
    /// The best position when trading 2 => 1.
    #[prost(message, optional, tag = "2")]
    pub best_2_to_1_position: ::core::option::Option<Position>,
    /// An approximation of the effective price when trading 1 => 2.
    #[prost(double, tag = "3")]
    pub approx_effective_price_1_to_2: f64,
    /// An approximation of the effective price when trading 2 => 1.
    #[prost(double, tag = "4")]
    pub approx_effective_price_2_to_1: f64,
}
impl ::prost::Name for SpreadResponse {
    const NAME: &'static str = "SpreadResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimulateTradeRequest {
    #[prost(message, optional, tag = "1")]
    pub input: ::core::option::Option<super::super::super::asset::v1alpha1::Value>,
    #[prost(message, optional, tag = "2")]
    pub output: ::core::option::Option<super::super::super::asset::v1alpha1::AssetId>,
    #[prost(message, optional, tag = "3")]
    pub routing: ::core::option::Option<simulate_trade_request::Routing>,
}
/// Nested message and enum types in `SimulateTradeRequest`.
pub mod simulate_trade_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Routing {
        #[prost(oneof = "routing::Setting", tags = "1, 2")]
        pub setting: ::core::option::Option<routing::Setting>,
    }
    /// Nested message and enum types in `Routing`.
    pub mod routing {
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct SingleHop {}
        impl ::prost::Name for SingleHop {
            const NAME: &'static str = "SingleHop";
            const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
            fn full_name() -> ::prost::alloc::string::String {
                ::prost::alloc::format!(
                    "penumbra.core.component.dex.v1alpha1.SimulateTradeRequest.Routing.{}",
                    Self::NAME
                )
            }
        }
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Default {}
        impl ::prost::Name for Default {
            const NAME: &'static str = "Default";
            const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
            fn full_name() -> ::prost::alloc::string::String {
                ::prost::alloc::format!(
                    "penumbra.core.component.dex.v1alpha1.SimulateTradeRequest.Routing.{}",
                    Self::NAME
                )
            }
        }
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Setting {
            #[prost(message, tag = "1")]
            Default(Default),
            #[prost(message, tag = "2")]
            SingleHop(SingleHop),
        }
    }
    impl ::prost::Name for Routing {
        const NAME: &'static str = "Routing";
        const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
        fn full_name() -> ::prost::alloc::string::String {
            ::prost::alloc::format!(
                "penumbra.core.component.dex.v1alpha1.SimulateTradeRequest.{}",
                Self::NAME
            )
        }
    }
}
impl ::prost::Name for SimulateTradeRequest {
    const NAME: &'static str = "SimulateTradeRequest";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimulateTradeResponse {
    #[prost(message, optional, tag = "1")]
    pub output: ::core::option::Option<SwapExecution>,
}
impl ::prost::Name for SimulateTradeResponse {
    const NAME: &'static str = "SimulateTradeResponse";
    const PACKAGE: &'static str = "penumbra.core.component.dex.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("penumbra.core.component.dex.v1alpha1.{}", Self::NAME)
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query operations for the DEX component.
    #[derive(Debug, Clone)]
    pub struct QueryServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Get the batch clearing prices for a specific block height and trading pair.
        pub async fn batch_swap_output_data(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchSwapOutputDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchSwapOutputDataResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/BatchSwapOutputData",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "BatchSwapOutputData",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Get the precise swap execution used for a specific batch swap.
        pub async fn swap_execution(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapExecutionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SwapExecutionResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/SwapExecution",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "SwapExecution",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Get the precise execution used to perform on-chain arbitrage.
        pub async fn arb_execution(
            &mut self,
            request: impl tonic::IntoRequest<super::ArbExecutionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ArbExecutionResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/ArbExecution",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "ArbExecution",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Stream all swap executions over a range of heights, optionally subscribing to future executions.
        pub async fn swap_executions(
            &mut self,
            request: impl tonic::IntoRequest<super::SwapExecutionsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::SwapExecutionsResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/SwapExecutions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "SwapExecutions",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Stream all arbitrage executions over a range of heights, optionally subscribing to future executions.
        pub async fn arb_executions(
            &mut self,
            request: impl tonic::IntoRequest<super::ArbExecutionsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ArbExecutionsResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/ArbExecutions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "ArbExecutions",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query all liquidity positions on the DEX.
        pub async fn liquidity_positions(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::LiquidityPositionsResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "LiquidityPositions",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query liquidity positions by ID.
        ///
        /// To get multiple positions, use `LiquidityPositionsById`.
        pub async fn liquidity_position_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::LiquidityPositionByIdResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositionById",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "LiquidityPositionById",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Query multiple liquidity positions by ID.
        pub async fn liquidity_positions_by_id(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionsByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<
                tonic::codec::Streaming<super::LiquidityPositionsByIdResponse>,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositionsById",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "LiquidityPositionsById",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Query liquidity positions on a specific pair, sorted by effective price.
        pub async fn liquidity_positions_by_price(
            &mut self,
            request: impl tonic::IntoRequest<super::LiquidityPositionsByPriceRequest>,
        ) -> std::result::Result<
            tonic::Response<
                tonic::codec::Streaming<super::LiquidityPositionsByPriceResponse>,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositionsByPrice",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "LiquidityPositionsByPrice",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        /// Get the current (direct) spread on a trading pair.
        ///
        /// This method doesn't do simulation, so actually executing might result in a
        /// better price (if the chain takes a different route to the target asset).
        pub async fn spread(
            &mut self,
            request: impl tonic::IntoRequest<super::SpreadRequest>,
        ) -> std::result::Result<tonic::Response<super::SpreadResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.QueryService/Spread",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.QueryService",
                        "Spread",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod simulation_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Simulation for the DEX component.
    ///
    /// This is a separate service from the QueryService because it's not just a
    /// simple read query from the state. Thus it poses greater DoS risks, and node
    /// operators may want to enable it separately.
    #[derive(Debug, Clone)]
    pub struct SimulationServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl SimulationServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> SimulationServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> SimulationServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            SimulationServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Simulate routing and trade execution.
        pub async fn simulate_trade(
            &mut self,
            request: impl tonic::IntoRequest<super::SimulateTradeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SimulateTradeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/penumbra.core.component.dex.v1alpha1.SimulationService/SimulateTrade",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "penumbra.core.component.dex.v1alpha1.SimulationService",
                        "SimulateTrade",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod query_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServiceServer.
    #[async_trait]
    pub trait QueryService: Send + Sync + 'static {
        /// Get the batch clearing prices for a specific block height and trading pair.
        async fn batch_swap_output_data(
            &self,
            request: tonic::Request<super::BatchSwapOutputDataRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchSwapOutputDataResponse>,
            tonic::Status,
        >;
        /// Get the precise swap execution used for a specific batch swap.
        async fn swap_execution(
            &self,
            request: tonic::Request<super::SwapExecutionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SwapExecutionResponse>,
            tonic::Status,
        >;
        /// Get the precise execution used to perform on-chain arbitrage.
        async fn arb_execution(
            &self,
            request: tonic::Request<super::ArbExecutionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ArbExecutionResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the SwapExecutions method.
        type SwapExecutionsStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::SwapExecutionsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Stream all swap executions over a range of heights, optionally subscribing to future executions.
        async fn swap_executions(
            &self,
            request: tonic::Request<super::SwapExecutionsRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::SwapExecutionsStream>,
            tonic::Status,
        >;
        /// Server streaming response type for the ArbExecutions method.
        type ArbExecutionsStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::ArbExecutionsResponse, tonic::Status>,
            >
            + Send
            + 'static;
        /// Stream all arbitrage executions over a range of heights, optionally subscribing to future executions.
        async fn arb_executions(
            &self,
            request: tonic::Request<super::ArbExecutionsRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::ArbExecutionsStream>,
            tonic::Status,
        >;
        /// Server streaming response type for the LiquidityPositions method.
        type LiquidityPositionsStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::LiquidityPositionsResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        /// Query all liquidity positions on the DEX.
        async fn liquidity_positions(
            &self,
            request: tonic::Request<super::LiquidityPositionsRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::LiquidityPositionsStream>,
            tonic::Status,
        >;
        /// Query liquidity positions by ID.
        ///
        /// To get multiple positions, use `LiquidityPositionsById`.
        async fn liquidity_position_by_id(
            &self,
            request: tonic::Request<super::LiquidityPositionByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<super::LiquidityPositionByIdResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the LiquidityPositionsById method.
        type LiquidityPositionsByIdStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::LiquidityPositionsByIdResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        /// Query multiple liquidity positions by ID.
        async fn liquidity_positions_by_id(
            &self,
            request: tonic::Request<super::LiquidityPositionsByIdRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::LiquidityPositionsByIdStream>,
            tonic::Status,
        >;
        /// Server streaming response type for the LiquidityPositionsByPrice method.
        type LiquidityPositionsByPriceStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<
                    super::LiquidityPositionsByPriceResponse,
                    tonic::Status,
                >,
            >
            + Send
            + 'static;
        /// Query liquidity positions on a specific pair, sorted by effective price.
        async fn liquidity_positions_by_price(
            &self,
            request: tonic::Request<super::LiquidityPositionsByPriceRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::LiquidityPositionsByPriceStream>,
            tonic::Status,
        >;
        /// Get the current (direct) spread on a trading pair.
        ///
        /// This method doesn't do simulation, so actually executing might result in a
        /// better price (if the chain takes a different route to the target asset).
        async fn spread(
            &self,
            request: tonic::Request<super::SpreadRequest>,
        ) -> std::result::Result<tonic::Response<super::SpreadResponse>, tonic::Status>;
    }
    /// Query operations for the DEX component.
    #[derive(Debug)]
    pub struct QueryServiceServer<T: QueryService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: QueryService> QueryServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServiceServer<T>
    where
        T: QueryService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.core.component.dex.v1alpha1.QueryService/BatchSwapOutputData" => {
                    #[allow(non_camel_case_types)]
                    struct BatchSwapOutputDataSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::BatchSwapOutputDataRequest>
                    for BatchSwapOutputDataSvc<T> {
                        type Response = super::BatchSwapOutputDataResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchSwapOutputDataRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::batch_swap_output_data(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = BatchSwapOutputDataSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/SwapExecution" => {
                    #[allow(non_camel_case_types)]
                    struct SwapExecutionSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::SwapExecutionRequest>
                    for SwapExecutionSvc<T> {
                        type Response = super::SwapExecutionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SwapExecutionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::swap_execution(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapExecutionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/ArbExecution" => {
                    #[allow(non_camel_case_types)]
                    struct ArbExecutionSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::ArbExecutionRequest>
                    for ArbExecutionSvc<T> {
                        type Response = super::ArbExecutionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ArbExecutionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::arb_execution(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ArbExecutionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/SwapExecutions" => {
                    #[allow(non_camel_case_types)]
                    struct SwapExecutionsSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<super::SwapExecutionsRequest>
                    for SwapExecutionsSvc<T> {
                        type Response = super::SwapExecutionsResponse;
                        type ResponseStream = T::SwapExecutionsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SwapExecutionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::swap_executions(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SwapExecutionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/ArbExecutions" => {
                    #[allow(non_camel_case_types)]
                    struct ArbExecutionsSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<super::ArbExecutionsRequest>
                    for ArbExecutionsSvc<T> {
                        type Response = super::ArbExecutionsResponse;
                        type ResponseStream = T::ArbExecutionsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ArbExecutionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::arb_executions(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ArbExecutionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositions" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionsSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<
                        super::LiquidityPositionsRequest,
                    > for LiquidityPositionsSvc<T> {
                        type Response = super::LiquidityPositionsResponse;
                        type ResponseStream = T::LiquidityPositionsStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LiquidityPositionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::liquidity_positions(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositionById" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionByIdSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::LiquidityPositionByIdRequest>
                    for LiquidityPositionByIdSvc<T> {
                        type Response = super::LiquidityPositionByIdResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LiquidityPositionByIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::liquidity_position_by_id(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionByIdSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositionsById" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionsByIdSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<
                        super::LiquidityPositionsByIdRequest,
                    > for LiquidityPositionsByIdSvc<T> {
                        type Response = super::LiquidityPositionsByIdResponse;
                        type ResponseStream = T::LiquidityPositionsByIdStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::LiquidityPositionsByIdRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::liquidity_positions_by_id(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionsByIdSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/LiquidityPositionsByPrice" => {
                    #[allow(non_camel_case_types)]
                    struct LiquidityPositionsByPriceSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::ServerStreamingService<
                        super::LiquidityPositionsByPriceRequest,
                    > for LiquidityPositionsByPriceSvc<T> {
                        type Response = super::LiquidityPositionsByPriceResponse;
                        type ResponseStream = T::LiquidityPositionsByPriceStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::LiquidityPositionsByPriceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::liquidity_positions_by_price(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = LiquidityPositionsByPriceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/penumbra.core.component.dex.v1alpha1.QueryService/Spread" => {
                    #[allow(non_camel_case_types)]
                    struct SpreadSvc<T: QueryService>(pub Arc<T>);
                    impl<
                        T: QueryService,
                    > tonic::server::UnaryService<super::SpreadRequest>
                    for SpreadSvc<T> {
                        type Response = super::SpreadResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SpreadRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QueryService>::spread(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SpreadSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: QueryService> Clone for QueryServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: QueryService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: QueryService> tonic::server::NamedService for QueryServiceServer<T> {
        const NAME: &'static str = "penumbra.core.component.dex.v1alpha1.QueryService";
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod simulation_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with SimulationServiceServer.
    #[async_trait]
    pub trait SimulationService: Send + Sync + 'static {
        /// Simulate routing and trade execution.
        async fn simulate_trade(
            &self,
            request: tonic::Request<super::SimulateTradeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SimulateTradeResponse>,
            tonic::Status,
        >;
    }
    /// Simulation for the DEX component.
    ///
    /// This is a separate service from the QueryService because it's not just a
    /// simple read query from the state. Thus it poses greater DoS risks, and node
    /// operators may want to enable it separately.
    #[derive(Debug)]
    pub struct SimulationServiceServer<T: SimulationService> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: SimulationService> SimulationServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SimulationServiceServer<T>
    where
        T: SimulationService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/penumbra.core.component.dex.v1alpha1.SimulationService/SimulateTrade" => {
                    #[allow(non_camel_case_types)]
                    struct SimulateTradeSvc<T: SimulationService>(pub Arc<T>);
                    impl<
                        T: SimulationService,
                    > tonic::server::UnaryService<super::SimulateTradeRequest>
                    for SimulateTradeSvc<T> {
                        type Response = super::SimulateTradeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SimulateTradeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as SimulationService>::simulate_trade(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SimulateTradeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: SimulationService> Clone for SimulationServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: SimulationService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: SimulationService> tonic::server::NamedService
    for SimulationServiceServer<T> {
        const NAME: &'static str = "penumbra.core.component.dex.v1alpha1.SimulationService";
    }
}
