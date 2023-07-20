#![forbid(unsafe_code)]

mod args;
pub mod error;
pub mod handler;

pub type CliResult = Result<(), error::CliError>;

/// A set of common imports you are likely to use.
pub mod prelude {
    pub use clap::clap_derive::*;
    pub use clap::*;

    pub use super::CliResult;
    pub use crate::error::CliError;
    pub use crate::handler::CliHandler;
}
