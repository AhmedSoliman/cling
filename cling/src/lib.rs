#![forbid(unsafe_code)]
//! # Cling
//!
//! Cling is a library that makes it easy to write command line applications in
//! Rust.

mod anymap;
mod app;
pub mod args;
mod command;
mod error;
mod extractors;
pub mod handler;

pub use app::{ClapClingExt, Cling};
pub use args::CliParam;
pub use command::CliRunnable;
pub use error::{CliError, CliErrorHandler};
pub use extractors::State;
pub use handler::IntoCliResult;

/// Convenience type alias for the result type of CLI applications
pub type CliResult = Result<(), error::CliError>;

/// Prelude module that contains most imports you'll need
///
/// This also imports clap, the following is a common pattern:
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
///     app.run_and_exit().await;
/// }
/// ```
pub mod prelude {
    pub use async_trait::async_trait;
    pub use clap::*;
    #[cfg(feature = "derive")]
    pub use cling_derive::*;

    pub use super::CliResult;
    pub use crate::app::*;
    pub use crate::command::CliRunnable;
    pub use crate::error::*;
    pub use crate::extractors::*;
    pub use crate::handler::CliHandler;
}
