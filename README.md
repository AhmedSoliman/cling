# cling
[WIP] An opinionated Rust framework for building multi-level command-line applications on top of clap.rs


## High-level direction
* Supports async Rust command-line applications and don't tie to specific async runtime.
* Promote specific design patterns with limited escape hatches
* Creates CLIs that follow design best practices by default
    * Support for configuration
    * Logging
    * Unit testable commands
    * Propagates errors to main with sane default printer
    * Translates Result::Err to non-zero exit codes
    * Support for building REPLs
    * Batteries included for output colorization, styling, and tty detection
    * Separates data from presentation layer (debatable)
    * Do we want to support non-derive clap as well? No!
