/*!
A Rust implementation of Lojban lujvo (compound word) generation and analysis.

# Examples

```rust
use vlazba::jvozba::{jvozba, tools::RafsiOptions};

let result = jvozba(&["klama".to_string(), "gasnu".to_string()], false, false, true, &RafsiOptions {
            exp_rafsi: false,
            custom_cmavo: None,
            custom_cmavo_exp: None,
            custom_gismu: None,
            custom_gismu_exp: None,
        });
assert!(result.iter().any(|r| r.lujvo == "klagau"));
```

```rust
use vlazba::jvokaha::jvokaha;

let decomposition = jvokaha("kalga'u").unwrap();
assert_eq!(decomposition, vec!["kal", "ga'u"]);
```
*/

pub mod cli_support;
pub mod error;
pub mod gismu;
pub mod morphology;

// --- Path aliases kept for lensisku / existing crates.io consumers ---

/// Alias of [`gismu`] — `vlazba::gismu_utils::GismuMatcher`.
pub use gismu as gismu_utils;

/// Alias of [`morphology`] — `vlazba::jvozba::{jvozba, tools, …}`.
pub use morphology as jvozba;

/// Alias of [`morphology::decompose`] — `vlazba::jvokaha::jvokaha`.
pub use morphology::decompose as jvokaha;

/// Alias of [`cli_support`] for the binary and older imports.
pub mod libs {
    pub use crate::cli_support::constants as config;

    #[cfg(feature = "cli")]
    pub mod cli {
        pub use crate::cli_support::weights::*;
    }
}

pub use error::{Result, VlazbaError};
pub use gismu::{GismuGenerator, GismuMatcher, GismuScorer};
pub use morphology::compound::{JvozbaOptions, LujvoAndScore, jvozba_with};
pub use morphology::lookup::{
    LujvoSpellingAnalysis, analyze_lujvo_spelling, get_candid, reconstruct_lujvo,
    search_selrafsi_from_rafsi2,
};
pub use morphology::score::get_lujvo_score;
