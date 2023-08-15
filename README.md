# cling
`cling` is a Rust framework that simplifies building command-line programs using [clap.rs](https://clap.rs).

>> Note: This project is in alpha stage and should not be used in production use cases. APIs can break without prior notice. Please provide feedback through github issues.

[![License](https://img.shields.io/badge/license-BSD--2--Clause--Patent-blue?style=flat-square
)](LICENSE)
[![Build status](https://github.com/AhmedSoliman/cling/actions/workflows/check.yml/badge.svg?branch=main)](https://github.com/AhmedSoliman/cling/actions/workflows/check.yml)
[![Crates.io](https://img.shields.io/crates/v/cling)](https://crates.io/crates/cling)
[![Documentation](https://docs.rs/cling/badge.svg)](https://docs.rs/cling)

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

# About
`cling` name is a play on CLI-ng (as in next-gen) and "cling" the english word, 
as it enables function handlers to _cling_ to [clap](https://clap.rs) user-defined structs 😉.

The handler design is inspired by [Axum](https://github.com/tokio-rs/axum), therefore, I like to think about `cling` as "Axum for command-line applications". 

For more details, see:
- [docs.rs](https://docs.rs/cling/latest/cling/)
- [examples](examples/)

# Example

```rust
use cling::prelude::*;

#[derive(CliRunnable, Parser, Debug, Clone)]
#[cling(run = "run")]
pub struct App {
    #[command(flatten)]
    pub options: Options,
}

// Structs that derive CliParam are optionally available for handlers as
// parameters both as value and reference.
#[derive(CliParam, Parser, Debug, Clone)]
pub struct Options {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

// handlers can be sync or async, cling will handle this transparently.
pub async fn run(options: &Options) {
    println!("Opts: {options:?}");
}

#[tokio::main]
async fn main() {
    let app = App::parse();
    app.run_and_exit().await;
}
```

Run it with:
```console
$ simple -d -d
Opts: Options { debug: 2 }

```

## Design direction
* Support both async and sync Rust command-line applications and don't tie to specific async runtime.
* Promote specific design patterns with limited escape hatches.
* Creates CLIs that follow design best practices by default:
    * Support for configuration
    * Logging
    * Unit testable commands
    * Propagates errors to main with sane default printer
    * Translates Result::Err to non-zero exit codes
    * Support for building REPLs
    * Do we want to support non-derive clap as well? No!

# Minimum Supported Rust Version (MSRV)
Cling's MSRV is 1.65.0.
