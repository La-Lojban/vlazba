//! Lojban morphology: lujvo generation, decomposition, rafsi lookup, and scoring.

pub mod compound;
pub mod cv;
pub mod decompose;
pub mod lookup;
pub mod rafsi_tables;
pub mod score;

// --- Public module aliases (lensisku / crates.io compatibility) ---

/// Compatibility alias for [`lookup`] (`vlazba::jvozba::tools::…`).
pub mod tools {
    pub use super::lookup::*;
}

/// Compatibility alias for [`decompose`] (`vlazba::jvozba::jvokaha` path).
pub mod jvokaha {
    pub use super::decompose::*;
}

pub use compound::{JvozbaOptions, LujvoAndScore, jvozba, jvozba_with, normalize};
pub use score::{get_lujvo_score, lujvo_score};
