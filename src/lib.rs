/*!
A Rust implementation of Lojban lujvo (compound word) generation and analysis.

# Examples

```rust
use vlazba::jvozba::{jvozba, LujvoAndScore, tools::RafsiOptions};

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
use vlazba::jvozba::jvokaha::jvokaha;

let decomposition = jvokaha("kalga'u").unwrap();
assert_eq!(decomposition, vec!["kal", "ga'u"]);
```
*/

pub mod gismu_utils;
pub mod jvozba;
pub mod libs;

pub use jvozba::{
    jvokaha,
    jvozba,
    scoring::get_lujvo_score,
    tools::{
        analyze_lujvo_spelling, get_candid, reconstruct_lujvo, search_selrafsi_from_rafsi2,
        LujvoSpellingAnalysis,
    },
};
pub use gismu_utils::{GismuGenerator, GismuMatcher, GismuScorer};
