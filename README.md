# vlazba

[![crates.io](https://img.shields.io/crates/v/vlazba.svg)](https://crates.io/crates/vlazba)
[![Docs](https://docs.rs/vlazba/badge.svg)](https://docs.rs/vlazba)

A Rust library and CLI for Lojban lujvo (compound word) generation and analysis.

Implements the gismu clash and jvozba algorithms described in [The Complete Lojban Language](https://lojban.github.io/cll/13/4/).

## Features

- Generates gismu based on input from transliterations of words in multiple languages
- Creates lujvo using the jvozba algorithm
- Customizable language weighting
- Efficient Rust implementation

## Installation

1. Ensure you have Rust installed and up to date:

   ```bash
   rustup default stable
   rustup update
   ```

2. Clone the repository and build the project:

   ```bash
   git clone https://github.com/la-lojban/vlazba.git
   cd vlazba
   cargo build --release
   ```

3. Install the CLI tool:
   ```bash
   cargo install vlazba --bin gimka
   ```

## As a Library

Add to your Cargo.toml:
```toml
[dependencies]
vlazba = "0.7"
```

Basic usage:
```rust
use vlazba::jvozba::{jvozba, LujvoAndScore};

 let results = jvozba(
     &["klama".to_string(), "gasnu".to_string()],
     false,
     false
 );
 assert!(results.iter().any(|r| r.lujvo == "klagau"));

// Analyze existing lujvo
let decomposition = jvokaha::jvokaha("kalga'u").unwrap();
```

## CLI Usage

### Gismu Generation

Basic usage:

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

To create lujvo using the jvozba algorithm:

```bash
./target/release/vlazba --jvozba "<word1> <word2> <word3>"
```

```bash
./target/release/vlazba --jvozba --exp-rafsi "<word1> <word2> <word3>"
```

Examples:

```bash
./target/release/vlazba --jvozba "klama klama gasnu"
```

```bash
./target/release/vlazba --jvozba --exp-rafsi "corci klama gasnu"
```

### Lujvo Reconstruction

To reconstruct a lujvo from its components:

```bash
./target/release/vlazba --reconstruct "<lujvo>"
```

Options:
- `--exp-rafsi`: Include experimental rafsi in reconstruction
- `--forbid-cmevla`: Forbid cmevla (name words) in reconstruction

Examples:

```bash
./target/release/vlazba --reconstruct "bramlatu"
./target/release/vlazba --reconstruct "bardymlatu" --exp-rafsi
./target/release/vlazba --reconstruct "toirbroda"
./target/release/vlazba --reconstruct "toirbroda" --forbid-cmevla
```

### Lujvo Decomposition (jvokaha)

To split lujvo using the jvokaha algorithm:

```bash
./target/release/vlazba --jvokaha "<lujvo>"
```

```bash
./target/release/vlazba --jvokaha --exp-rafsi "<lujvo>"
```

Examples:

```bash
./target/release/vlazba --jvokaha "klaklagau"
```

```bash
./target/release/vlazba --jvokaha --exp-rafsi "cocklagau"
```

## Options

- `-w, --weights`: Specify custom language weights (default: 0.347,0.196,0.160,0.123,0.089,0.085)
- `-s, --shapes`: Define gismu candidate shapes (default: "ccvcv,cvccv")
- `-a, --all-letters`: Use all available letters instead of only those in input words
- `-d, --deduplicate`: Path to existing gismu list for deduplication
- `--jvozba`: Use jvozba function to create lujvo instead of gismu generation
- `--forbid-la-lai-doi`: Forbid 'la', 'lai', 'doi' in lujvo when using jvozba
- `--jvokaha`: Use jvokaha function to split lujvo into components
- `--exp-rafsi`: Include experimental rafsi when generating lujvo

## Debug

```bash
RUST_BACKTRACE=full cargo run -- "uan rakan ekspekt esper predpologa mulud"
```

## Background

This project is a Rust rewrite of the original [gimyzba](https://github.com/teleological/gimyzba) and its [Python port](https://github.com/lynn/gimyzba). It aims to provide a more efficient and maintainable implementation of the gismu generation algorithm. Additionally it ports [jvozba](https://github.com/sozysozbot/sozysozbot_jvozba/tree/master) algorithm for getting lujvo creation functionality.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [GNU GENERAL PUBLIC LICENSE](LICENSE).
