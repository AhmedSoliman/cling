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

While writing a command-line applications using the terrific [clap](https://clap.rs)
crate, developers often write boilerplate functions that finds out which command the
user has executed, collects the input types, then runs a handler function that does
that job. This quickly gets repetitive for multi-command applications. Cling is designed
to simplify that workflow and lets developers declaratively map commands to function.

## Key Features
- Map CLI commands to handlers declaratively via `#[cling(run = "my_function")]`
- Define handlers as regular Rust unit-testable functions
- Run handlers on any level of the command tree (middlware-style)
- Handler function arguments are extracted automatically from clap input
- Handlers can return a [`State<T>`] value that can be extracted by downstream handlers
- Handlers can be either `sync` or `async` functions
- Uniform CLI-friendly error handling with colours

<div class="example-wrap" style="display:inline-block"><pre class="compile_fail" style="white-space:normal;font:inherit;">
Credit: The handler design of cling is largely inspired by the excellent work done in [Axum](https://github.com/tokio-rs/axum).
</pre></div>

For more details, see:
- [docs.rs](https://docs.rs/cling/latest/cling/)
- [examples](examples/)

*Compiler support: [requires `rustc` 1.72+][msrv]*

[msrv]: #supported-rust-versions

## Demo
A quick-and-dirty example to show cling in action

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
async fn beep() -> anyhow::Result<()> {
    println!("Beep, Beep!");
    Ok(())
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

# Concepts
1. [Runnables](#runnables)
2. [Handlers](#handlers)

## Runnables
Runnables refer to your clap structs that represent a CLI command. In cling,
any struct or enum that encodes the command tree must derive [`Run`], this
includes the top-level struct of your program (e.g. `MyApp` in the above demo).
The [`Run`] trait tells cling that this type should be attached to a [handler](#handlers)
function. Essentially, structs that derive [`Parser`] or [`Subcommand`] will need to derive [`Run`].

For any struct/enum that derive [`Run`], a `#[cling(run = ...)]` attribute can be used to
associate a [handler](#handler) with it. It's possible to design a multi-level clap
program with handlers that run on each level, let's look at a few examples to understand
what's possible.

### Single Command, Single Handler
```rust, no_run
use cling::prelude::*;

#[derive(Run, Collect, Parser, Debug, Clone)]
// standard clap attributes
#[command(author, version, about)]
#[cling(run = "my_only_handler")]
pub struct MyApp {
    #[clap(short, long)]
    /// User name
    pub name: String,
}

fn my_only_handler(app: &MyApp) {
    println!("User is {}", app.name);
}

#[tokio::main]
async fn main() -> ClingFinished<MyApp> {
    Cling::parse_and_run().await
}

```
Note: We derive [`Collect`] and [`Clone`] on `MyApp` in order to pass it down to 
handler `my_only_handler` as shared reference `app: &MyApp`. Deriving [`Collect`]
is not necessary if your handler doesn't need access to `&MyApp`.

Our program will now run the code in `my_only_handler`
```txt
$ sample-1 --name=Steve
User is Steve
```

### Sub-commands with handlers
Given a command structure that looks like this:
```txt
MyApp [CommonOpts]
  - projects
    |- create [CreateOpts]
    |- list
  - whoami
```
```rust,no_run
use cling::prelude::*;

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct MyApp {
    #[clap(flatten)]
    pub common: CommonOpts,
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Args, Collect, Debug, Clone)]
pub struct CommonOpts {
    /// Access token
    #[arg(long, global = true)]
    pub access_token: Option<String>,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum Commands {
    /// Manage projects
    #[command(subcommand)]
    Projects(ProjectCommands),
    /// Self identification
    #[cling(run = "handlers::whoami")]
    WhoAmI,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    /// Create new project
    Create(CreateOpts),
    /// List all projects
    #[cling(run = "handlers::list_projects")]
    List,
}

#[derive(Run, Args, Collect, Debug, Clone)]
#[cling(run = "handlers::create_project")]
pub struct CreateOpts {
    /// Project name
    pub name: String,
}

mod handlers {
    pub fn whoami() {}
    pub fn list_projects() {}
    pub fn create_project() {}
}

```
- `CommonOpts` Program-wide options. Any handler should be able to access it if it chooses to.
- `ProjectOpts` Options for `projects [OPTIONS]`
- `CreateOpts` Options for the `projects create` command
- `ListOpts` Options for the `projects list` command

To understand how this works, consider the different set of options that can be passed in `projects list` subcommand:

```txt
Usage: myapp [COMMON-OPTIONS] projects create <NAME>
```

Our runnables in this design are: `MyApp`, `Commands`, `ProjectCommands`, and `CreateOpts`.

We can attach a handler to any [`Run`] type by using `#[cling(run = "...")]` attribute. 
This handler will run _before_ any handler of sub-commands (if any). For enums that implement
[`Subcommand`], we **must** attach `#[cling(run = "...")]` on enum variants that do not 
take any arguments (like `ProjectCommands::List`). However, for any enum variant that takes
arguments, the argument type itself **must** derive [`Run`].

## Handlers
Handlers are regular Rust functions that get executed when a clap command is run. Handlers latch
onto any type that derive [`Run`] using the `#[cling(run = "function::path")]` attribute.

Every type that has `#[cling(run = ...)]` will run its handler function before running the handlers
of the inner runnables.

Example
```rust,no_run
use cling::prelude::*;

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[cling(run = "init")]
pub struct MyApp {
    #[clap(flatten)]
    pub common: CommonOpts,
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Args, Collect, Debug, Clone)]
pub struct CommonOpts {
    /// Access token
    #[arg(long, global = true)]
    pub access_token: Option<String>,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum Commands {
    #[cling(run = "run_beep")]
    Beep,
    /// Self identification
    #[cling(run = "run_whoami")]
    WhoAmI,
}

fn init() {
    println!("init handler!");
}

fn run_whoami() {
    println!("I'm groot!");
}

fn run_beep() {
    println!("Beep beep!");
}

#[tokio::main]
async fn main() -> ClingFinished<MyApp> {
    Cling::parse_and_run().await
}
```

`init` handler will always run before any other handlers in that structure.

```console
$ many-handlers beep
init handler!
Beep beep!

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

[`Run`]: crate::Run
[`Collect`]: crate::prelude::Collect
[`Parser`]: crate::prelude::Parser
[`Subcommand`]: crate::prelude::Subcommand

