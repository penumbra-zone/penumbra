pub mod output_view;
pub mod spend_view;
pub mod swap_claim_view;
pub mod swap_view;

pub use output_view::OutputView;
pub use spend_view::SpendView;
pub use swap_claim_view::SwapClaimView;
pub use swap_view::SwapView;

pub enum ActionView {
    /// View of a Swap action
    Swap(SwapView),
    SwapClaim(SwapClaimView),
    Output(OutputView),
    Spend(SpendView),
}
