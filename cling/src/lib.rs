#![forbid(unsafe_code)]
//! # Cling
//!
//! Cling is a framework that makes it easy to write command line applications
//! in Rust.
//!
//! Example:
//! ```
//! use cling::prelude::*;
//!
//! #[derive(CliRunnable, Parser, Debug, Clone)]
//! #[cling(run = "run")]
//! pub struct App {
//!     /// Turn debugging information on
//!     #[arg(short, long, action = ArgAction::Count)]
//!     pub debug: u8,
//! }
//!
//!
//! pub async fn run() {
//!     println!("Hello, world!");
//! }
//!
//! #[tokio::main]
//! async fn main() -> ClingFinished<App> {
//!     Cling::parse_and_run().await
//! }

mod anymap;
mod app;
mod error;
mod extractors;
mod handler;
mod params;

pub use app::*;
/// Derive a clap struct/enum to be passed as a handler parameter.
///
/// Typically, this is implemented on your types by using the
/// `#[derive(CliParam)]` attribute.
///
/// **Note:** _Types that implement [CliParam] must also
/// be [Clone]._
///
/// ## Example:
/// ```rust
/// use cling::prelude::*;
/// // Structs that derive CliParam are optionally available for handlers as
/// // parameters both as value and reference.
/// #[derive(CliParam, Parser, Debug, Clone)]
/// pub struct Options {
///     /// Turn debugging information on
///     #[arg(short, long, action = clap::ArgAction::Count)]
///     pub debug: u8,
/// }
/// ```
#[cfg(feature = "derive")]
pub use cling_derive::CliParam;
/// Derive clap structs as cling runnable command.
///
/// This trait needs to be derived for clap structs or enums that will run
/// a function handler or a if has subcommands. Note that cling requires
/// that all clap structs/enums implement [Clone].
///
/// Usually, this will be derived like the following example:
///
/// ```rust
/// use cling::prelude::*;
///
/// #[derive(CliRunnable, Parser, Debug, Clone)]
/// #[cling(run = "do_nothing")]
/// pub struct App {
///     /// Turn debugging information on
///     #[arg(short, long, action = ArgAction::Count)]
///     pub debug: u8,
/// }
///
/// fn do_nothing() {}
/// ```
///
/// Runnable structs will execute the handler specified in `#[cling(run =
/// "...")]` The string value must be a valid path to a function.
///
/// Handler functions can be async or sync, cling will handle this
/// transparently. However, Cling only supports async on the top level,
/// you'll need to pick an async runtime to execute the application.
#[cfg(feature = "derive")]
pub use cling_derive::CliRunnable;
pub use error::{CliError, CliErrorHandler};
pub use extractors::{Collected, State};
pub use handler::IntoCliResult;
pub use params::CliParam;

#[doc(hidden)]
/// Used by cling_derive
pub mod _private {
    pub use {static_assertions, tracing};

    pub use crate::handler::*;
    pub use crate::params::*;
}

/// Prelude module that contains most imports you'll need
///
/// This also imports clap but your program will need to have a dependency on
/// clap in Cargo.toml.
///
/// The following is a common pattern:
/// ```rust
/// use cling::prelude::*;
///
/// #[derive(CliRunnable, Parser, Debug, Clone)]
/// #[cling(run = "run")]
/// pub struct App {
///     /// Turn debugging information on
///     #[arg(short, long, action = ArgAction::Count)]
///     pub debug: u8,
/// }
///
/// pub async fn run() {
///     println!("Hello, world!");
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let app = App::parse();
///     let app = app.into_cling();
///     app.run_and_exit().await;
/// }
/// ```
pub mod prelude {
    pub use async_trait::async_trait;
    pub use clap::*;
    #[cfg(feature = "derive")]
    pub use cling_derive::*;

    pub use crate::app::*;
    pub use crate::error::*;
    pub use crate::extractors::*;
    pub use crate::handler::CliHandler;
}
