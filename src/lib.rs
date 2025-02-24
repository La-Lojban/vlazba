/*!
A Rust implementation of Lojban lujvo (compound word) generation and analysis.

# Examples

```rust
use vlazba::jvozba::{jvozba, LujvoAndScore};

let result = jvozba(&["klama".to_string(), "gasnu".to_string()], false, false, false);
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
    tools::{get_candid, search_selrafsi_from_rafsi2, reconstruct_lujvo},
};
