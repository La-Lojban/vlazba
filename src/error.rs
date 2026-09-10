use thiserror::Error;

/// Library error type for morphology failures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VlazbaError {
    #[error("You need at least two valsi to make a lujvo")]
    TooFewValsi,

    #[error("Failed to decompose {{{0}}}")]
    Decompose(String),

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Could not resolve rafsi `{rafsi}` in `{lujvo}`")]
    UnresolvedRafsi { rafsi: String, lujvo: String },

    #[error("Need at least two selrafsi to rebuild lujvo")]
    TooFewSelrafsi,

    #[error("Failed to rebuild lujvo")]
    RebuildFailed,
}

/// Convenient result alias for library APIs.
pub type Result<T> = std::result::Result<T, VlazbaError>;
