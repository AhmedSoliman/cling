// Enables unstable features on nightly/beta rustc.
#![cfg_attr(unstable, feature(marker_trait_attr))]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod anymap;
mod app;
mod effects;
mod error;
mod extractors;
mod handler;
mod params;

pub use app::*;
#[cfg(feature = "derive")]
/// Macro that adds a few assertions to help you investigate
/// errors if the compiler is not happy about a handler signature.
///
/// ## Example:
/// ```rust,no_run
/// use cling::prelude::*;
/// # #[derive(Clone)]
/// # pub struct Session {
/// #     user_id: String,
/// # }
/// # #[derive(Collect, Args, Debug, Clone)]
/// # pub struct CreateProjectArgs {
/// #     /// Name of the project
/// #     pub name: String,
/// # }
///
/// #[cling_handler]
/// pub async fn create_project(
///     State(session): State<Session>,
///     args: &CreateProjectArgs,
/// ) -> anyhow::Result<()> {
///     println!(
///         "Creating project '{}' for user {}.",
///         args.name, session.user_id
///     );
///     println!("Would have created the project here");
///     Ok(())
/// }
/// ```
pub use cling_derive::cling_handler;
#[cfg(feature = "derive")]
/// Mark a clap struct/enum to be passed as a handler argument.
///
/// **Note:** _Types that implement [Collect] must also be [Clone]._
///
/// ## Example:
/// ```rust
/// use cling::prelude::*;
/// #[derive(Collect, Args, Debug, Clone)]
/// pub struct Options {
///     /// Turn debugging information on
///     #[arg(short, long, action = clap::ArgAction::Count)]
///     pub debug: u8,
/// }
///
/// // Options must derive `Collect`
/// fn run(options: &Options) {
///    println!("Debug level is {}", options.debug);
/// }
/// ```
pub use cling_derive::Collect;
#[cfg(feature = "derive")]
/// Mark clap structs as cling runnable command.
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
/// #[derive(Run, Parser, Debug, Clone)]
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
/// [Run] types will execute the handler specified in `#[cling(run =
/// "...")]` The string value must be a valid path to a function.
///
/// Handler functions can be async or sync, cling will handle this
/// transparently. However, Cling only supports async on the top level,
/// you'll need to pick an async runtime to execute the application.
pub use cling_derive::Run;
pub use effects::{IntoEffect, SetState};
pub use error::{CliError, CliErrorHandler};
pub use extractors::{Collected, State};

#[doc(hidden)]
/// Used by cling_derive
pub mod _private {
    pub use {static_assertions, tracing};

    pub use crate::effects::*;
    pub use crate::handler::*;
    pub use crate::params::*;
}

/// Prelude module that contains most imports you'll need
///
/// This also imports clap but your program will need to have a dependency on
/// clap in Cargo.toml.
pub mod prelude {
    pub use clap::*;
    #[cfg(feature = "derive")]
    #[doc(no_inline)]
    pub use cling_derive::{cling_handler, Collect, Run};

    pub use crate::app::*;
    pub use crate::error::*;
    pub use crate::extractors::*;
    pub use crate::handler::Handler;
}
