use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum ProtoError {
    #[error("OutputBody is malformed")]
    OutputBodyMalformed,
    #[error("Output is malformed")]
    OutputMalformed,
    #[error("Spend is malformed")]
    SpendMalformed,
    #[error("SpendBody is malformed")]
    SpendBodyMalformed,
    #[error("Action malformed")]
    ActionMalformed,
}
