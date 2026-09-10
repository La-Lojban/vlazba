# vlazba

[![crates.io](https://img.shields.io/crates/v/vlazba.svg)](https://crates.io/crates/vlazba)
[![Docs](https://docs.rs/vlazba/badge.svg)](https://docs.rs/vlazba)

A Rust library and CLI for Lojban lujvo (compound word) generation and analysis.

Implements the gismu clash and jvozba algorithms described in [The Complete Lojban Language](https://lojban.github.io/cll/13/4/).

## Features

- Generates gismu based on input from transliterations of words in multiple languages
- Creates lujvo using the jvozba algorithm (full enumeration or score-optimal DP via `best_only`)
- Decomposes and reconstructs classical lujvo (`jvokaha`, `reconstruct_lujvo`)
- Customizable language weighting and custom rafsi maps
- Optional `cli` / `parallel` Cargo features for lean library consumers

## Installation

1. Ensure you have Rust 1.85+ installed:

   ```bash
   rustup default stable
   rustup update
   ```

2. Clone and build (CLI needs the default `cli` feature):

   ```bash
   git clone https://github.com/la-lojban/vlazba.git
   cd vlazba
   cargo build --release
   ```

3. Install the CLI tool:

   ```bash
   cargo install vlazba --features cli
   ```

## Module layout (idiomatic names)

| Preferred | Compatibility alias (lensisku) |
|-----------|--------------------------------|
| `gismu` | `gismu_utils` |
| `morphology` | `jvozba` |
| `morphology::decompose` | `jvokaha` |
| `morphology::lookup` | `jvozba::tools` |

Internal helpers use names like `rafsi_candidates`, `resolve_selrafsi`, `cartesian_product`, `cv_shape`, `lujvo_score`; legacy names remain as aliases.

## As a Library

Add to your Cargo.toml:

```toml
[dependencies]
vlazba = "1.0"

# Lean dependency (no clap):
# vlazba = { version = "1.0", default-features = false, features = ["parallel"] }
```

Basic usage:

```rust
use vlazba::jvozba::{jvozba, jvokaha, tools::RafsiOptions};

let results = jvozba(
    &["klama".to_string(), "gasnu".to_string()],
    false,
    false,
    true, // best_only: DP search for score-optimal forms only
    &RafsiOptions {
        exp_rafsi: false,
        custom_cmavo: None,
        custom_cmavo_exp: None,
        custom_gismu: None,
        custom_gismu_exp: None,
    },
);
assert!(results.iter().any(|r| r.lujvo == "klagau"));

let decomposition = jvokaha("kalga'u").unwrap();
```

Prefer `best_only = true` whenever you only need the score-optimal spelling (e.g. reconstruct / spelling analysis).

## CLI Usage

### Gismu Generation

```bash
./target/release/vlazba "<Mandarin> <Hindi> <English> <Spanish> <Russian> <Arabic>"
```

Example:

```bash
./target/release/vlazba "uan rakan ekspekt esper predpologa mulud"
```

Custom weights:

```bash
./target/release/vlazba -w 0.271,0.170,0.130,0.125,0.104,0.076,0.064,0.060 mandarin english spanish hindi arabic bengali russian portuguese
```

### Lujvo Creation (jvozba)

```bash
./target/release/vlazba --jvozba "klama klama gasnu"
./target/release/vlazba --jvozba --best-only "klama gasnu"
./target/release/vlazba --jvozba --exp-rafsi "corci klama gasnu"
```

### Lujvo Reconstruction

```bash
./target/release/vlazba --reconstruct "bramlatu"
./target/release/vlazba --reconstruct "bardymlatu" --exp-rafsi
./target/release/vlazba --reconstruct "toirbroda" --forbid-cmevla
```

### Lujvo Decomposition (jvokaha)

```bash
./target/release/vlazba --jvokaha "klaklagau"
./target/release/vlazba --jvokaha --exp-rafsi "cocklagau"
```

## Options

- `-w, --weights`: Custom language weights (default: 1985 set) or a year preset (`1995`, …)
- `-s, --shapes`: Gismu candidate shapes (default: `ccvcv,cvccv`)
- `-a, --all-letters`: Use all letters instead of only those in input words
- `-d, --deduplicate`: Path to existing gismu list for deduplication
- `--jvozba`: Create lujvo instead of gismu generation
- `--best-only`: With `--jvozba`, only score-optimal lujvo (DP)
- `--forbid-la-lai-doi`: Forbid `la` / `lai` / `doi` in cmevla-shaped lujvo
- `--jvokaha`: Split lujvo into components
- `--exp-rafsi`: Include experimental rafsi
- `--reconstruct` / `--forbid-cmevla`: Reconstruct / forbid cmevla forms

## Debug

```bash
RUST_BACKTRACE=full cargo run --features cli -- "uan rakan ekspekt esper predpologa mulud"
```

## Background

This project is a Rust rewrite of the original [gimyzba](https://github.com/teleological/gimyzba) and its [Python port](https://github.com/lynn/gimyzba). It also ports the [jvozba](https://github.com/sozysozbot/sozysozbot_jvozba/tree/master) algorithm for lujvo creation.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [GNU GENERAL PUBLIC LICENSE](LICENSE).
