# cling
`cling` is a Rust framework that simplifies building command-line programs using [clap.rs](https://clap.rs).

>> Note: This project is in alpha stage and should not be used in production use cases. APIs can break without prior notice.

[![License](https://img.shields.io/badge/license-BSD--2--Clause--Patent-blue?style=flat-square
)](LICENSE)
[![Build status](https://github.com/AhmedSoliman/cling/actions/workflows/check.yml/badge.svg?branch=main)](https://github.com/AhmedSoliman/cling/actions/workflows/check.yml)
[![Crates.io](https://img.shields.io/crates/v/cling)](https://crates.io/crates/cling)
[![Documentation](https://docs.rs/cling/badge.svg)](https://docs.rs/cling)

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).

# About

>> TODO

For more details, see:
- [docs.rs](https://docs.rs/cling/latest/cling/)
- [examples](examples/)

## High-level direction
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
