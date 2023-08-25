use std::fmt::{self, Display, Formatter};
use std::io::Write;

use clap::CommandFactory;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::prelude::ClingFinished;
use crate::CliRunnable;

pub trait CliErrorHandler {
    type Output;
    fn unwrap_or_exit(self) -> Self::Output;
    fn then_exit(self) -> !;
}

/// An error type for the CLI application.
///
/// This error type handles exit codes, pretty printing of error messages, and
/// include some handy utilities.
pub enum CliError {
    InvalidHandler(String),
    Failed,
    FailedWithMessage(String),
    FailedWithMessageAndCode(String, u8),
    ClapError(clap::Error),
    InputString,
    Other(anyhow::Error),
    OtherWithCode(anyhow::Error, u8),
}

impl std::error::Error for CliError {}

/// A helper to allow anyhow users from returning anyhow::Error errors in cling
/// handlers without writing an [[`Into<CliError>`]] implementation for every
/// error type.
impl From<anyhow::Error> for CliError {
    fn from(value: anyhow::Error) -> Self {
        CliError::Other(value)
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        CliError::Other(value.into())
    }
}

impl From<clap::Error> for CliError {
    fn from(value: clap::Error) -> Self {
        CliError::ClapError(value)
    }
}

impl<T, E> CliErrorHandler for Result<T, E>
where
    E: Into<CliError>,
{
    type Output = T;

    /// Returns the result if it is `Ok`, otherwise exit the program with the
    /// appropriate exit code after printing the error.
    fn unwrap_or_exit(self) -> T {
        match self {
            | Ok(x) => x,
            | Err(e) => {
                let e = e.into();
                e.print().unwrap();
                e.exit()
            }
        }
    }

    /// Exit the program with appropriate exit code. This will also print the
    /// error if the result is an error.
    fn then_exit(self) -> ! {
        match self {
            | Ok(_) => std::process::exit(0),
            | Err(e) => {
                let e = e.into();
                e.print().unwrap();
                e.exit()
            }
        }
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            | CliError::ClapError(e) => {
                // Clap handles colors
                write!(f, "{}", e)
            }
            | CliError::Failed => {
                write!(f, "Failed!")
            }
            | CliError::FailedWithMessage(e) => {
                write!(f, "Failed: {}", e)
            }
            | CliError::FailedWithMessageAndCode(e, _) => {
                write!(f, "Error: {}", e)
            }
            | CliError::Other(e) => {
                write!(f, "Error: {}", e)
            }
            | CliError::OtherWithCode(e, _) => {
                write!(f, "Error: {}", e)
            }
            | CliError::InputString => {
                write!(f, "Input string cannot be parsed as UNIX shell command")
            }
            | CliError::InvalidHandler(msg) => {
                write!(f, "\n\n** Cling Handler Design Error **\n\n{}", msg)
            }
        }
    }
}

impl std::fmt::Debug for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl CliError {
    /// Pretty print the error to stderr.
    pub fn print(&self) -> std::io::Result<()> {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
        match self {
            | CliError::ClapError(e) => {
                // Clap handles colors
                e.print()
            }
            | CliError::Failed => {
                print_formatted_error(&mut stderr, "Aborted!", "")
            }
            | CliError::FailedWithMessage(e) => {
                print_formatted_error(&mut stderr, "", e)
            }
            | CliError::FailedWithMessageAndCode(e, _) => {
                print_formatted_error(&mut stderr, "", e)
            }
            | CliError::Other(e) => {
                print_formatted_error(&mut stderr, "Error: ", &e.to_string())
            }
            | CliError::OtherWithCode(e, _) => {
                print_formatted_error(&mut stderr, "Error: ", &e.to_string())
            }
            | e @ CliError::InputString => {
                print_formatted_error(&mut stderr, "", &e.to_string())
            }
            | CliError::InvalidHandler(msg) => {
                print_formatted_error(
                    &mut stderr,
                    "\n\n** Cling Handler Design Error **\n\n",
                    msg,
                )
            }
        }
    }

    /// What is the exit code for this error?
    pub fn exit_code(&self) -> u8 {
        match self {
            | CliError::FailedWithMessageAndCode(_, code) => *code,
            | CliError::OtherWithCode(_, code) => *code,
            // Clap uses i32 for exit codes, we cast to u8 but fail with 255 if
            // out of bound.
            | CliError::ClapError(e) => {
                let code = e.exit_code();
                code.try_into().unwrap_or(255)
            }
            | _ => 1,
        }
    }

    /// Terminate the program with this error's exit code.
    pub fn exit(self) -> ! {
        std::process::exit(self.exit_code() as i32)
    }

    pub fn into_finished<T: CliRunnable + clap::Parser>(
        self,
    ) -> ClingFinished<T> {
        Into::into(self)
    }
}

static_assertions::assert_impl_all!(CliError: Send, Sync);

fn print_formatted_error(
    f: &mut StandardStream,
    heading: &str,
    msg: &str,
) -> std::io::Result<()> {
    f.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
    write!(f, "{}", heading)?;
    f.reset()?;
    writeln!(f, "{}", msg)?;
    Ok(())
}

pub(crate) fn format_clap_error<I: CommandFactory>(
    err: clap::Error,
) -> clap::Error {
    let mut cmd = I::command();
    err.format(&mut cmd)
}
