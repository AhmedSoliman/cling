#![forbid(unsafe_code)]

mod anymap;
mod app;
pub mod args;
mod command;
pub mod error;
mod extractors;
pub mod handler;

pub use static_assertions::assert_impl_all;

pub type CliResult = Result<(), error::CliError>;
pub use ::clap;
pub use async_trait::async_trait;
/// A set of common imports you are likely to use.
pub mod prelude {
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
