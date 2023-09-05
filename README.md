A lightweight framework that simplifies building complex command-line applications with [clap.rs](https://clap.rs).

[![license-badge][]][license]
[![build-status-badge][]][build-status]
[![crates-io-badge][]][crates-io]
[![docs-badge][]][docs]

[license-badge]: https://img.shields.io/badge/license-BSD--2--Clause--Patent-blue?style=flat-square
[license]: #license
[build-status-badge]: https://github.com/AhmedSoliman/cling/actions/workflows/check.yml/badge.svg?branch=main
[build-status]: https://github.com/AhmedSoliman/cling/actions/workflows/check.yml?query=branch%3Amain
[crates-io-badge]: https://img.shields.io/crates/v/cling
[crates-io]: https://crates.io/crates/cling
[docs-badge]: https://docs.rs/cling/badge.svg
[docs]: https://docs.rs/cling

# Overview

`cling`'s name is a play on CLI-ng (as in next-gen) and "cling" the English word. It enables function handlers to _cling_ to [clap](https://clap.rs) user-defined structs 😉.

While writing a command-line application using the terrific [clap](https://clap.rs), developers often write some boilerplate code to find which command the user has executed, collect the input parameter values, and run a handler function that does that job. This quickly gets repetitive for multi-command or multi-level command-line applications. Cling lets you declaratively map functions to clap's structs, and it'll handle all the plumbing for you!

## Key Features
- Connects CLI commands to handlers declaratively via `#[cling(run = "my_function")]`.
- Define handlers as normal unit-testable functions.
- Run handlers on any level of the command tree (middlware-style)
- Handler function arguments are extracted automatically from clap input.
- Handlers can return a [State<T>] value that can be extracted by downstream handlers
- Handlers can be either `sync` or `async` functions.
- Uniform CLI-friendly error handling with colors

<div class="example-wrap" style="display:inline-block"><pre class="compile_fail" style="white-space:normal;font:inherit;">
**Credit:** _The handler design of cling is largely inspired by the excellent work done in [Axum](https://github.com/tokio-rs/axum)._
</pre></div>

For more details, see:
- [docs.rs](https://docs.rs/cling/latest/cling/)
- [examples](examples/)

*Compiler support: [requires `rustc` 1.70+][msrv]*



[msrv]: #supported-rust-versions



## Demo
A quick-and-dirty example to show cling in action might look like the following:

```toml
# Cargo.toml
[package]
name = "cling-example"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.4.1", features = ["derive", "env"] }
tokio = { version = "1.13.0", features = ["full"] }
```

Our `main.rs` might look something like this:
```rust, no_run
use cling::prelude::*;

// -- args --

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, long_about = None)]
/// A simple multi-command example using cling
struct MyApp {
    #[clap(flatten)]
    pub opts: CommonOpts,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Collect, Parser, Debug, Clone)]
struct CommonOpts {
    /// Turn debugging information on
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

// Commands for the app are defined here.
#[derive(Run, Subcommand, Debug, Clone)]
enum Command {
    /// Honk the horn!
    Honk(HonkOpts),
    #[cling(run = "beep")]
    /// Beep beep!
    Beep,
}

// Options of "honk" command. We define cling(run=...) here to call the
// function when this command is executed.
#[derive(Run, Collect, Parser, Debug, Clone)]
#[cling(run = "honk")]
struct HonkOpts {
    /// How many times to honk?
    times: u8,
}

// -- Handlers --

// We can access &CommonOpts because it derives [Collect]
fn honk(common_opts: &CommonOpts, HonkOpts { times }: &HonkOpts) {
    if common_opts.verbose > 0 {
        println!("Honking {} times", times);
    }

    (0..*times).for_each(|_| {
        print!("Honk ");
    });
    println!("!");
}

// Maybe beeps need to be async!
async fn beep() {
    println!("Beep, Beep!");
}


// -- main --

#[tokio::main]
async fn main() -> ClingFinished<MyApp> {
    Cling::parse_and_run().await
}
```

Now, let's run it and verify that it works as expected
```console
$ simple-multi-command -v honk 5
Honking 5 times
Honk Honk Honk Honk Honk !

```

# Feature Flags

| Feature  | Activation         | Effect
|----------|--------------------|--------
| `derive` | default            | Enables `#[derive(Run)]` and `#[derive(Collect)]`
| `shlex`  | "shlex" feature    | Enables parsing from text, useful when building REPLs

# Supported Rust Versions

Cling's minimum supported rust version is `1.70.0`.

# License

Dual-licensed under [Apache 2.0](https://github.com/AhmedSoliman/cling/blob/main/LICENSE-APACHE) or [MIT](https://github.com/AhmedSoliman/cling/blob/main/LICENSE-MIT).

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
