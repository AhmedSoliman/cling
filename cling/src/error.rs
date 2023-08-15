use std::io::Write;

use clap::CommandFactory;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use thiserror::Error;

pub trait CliErrorHandler {
    type Output;
    fn unwrap_or_exit(self) -> Self::Output;
    fn then_exit(self) -> !;
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid Handler: {0}")]
    InvalidHandler(String),
    #[error("Aborted!")]
    Abort,
    #[error("Aborted: {0}")]
    AbortMessage(String),
    #[error(transparent)]
    ClapError(#[from] clap::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl<T, E> CliErrorHandler for Result<T, E>
where
    E: Into<CliError>,
{
    type Output = T;

    fn unwrap_or_exit(self) -> T {
        match self {
            | Ok(x) => x,
            | Err(e) => {
                print_err(e);
                std::process::exit(1);
            }
        }
    }

    fn then_exit(self) -> ! {
        match self {
            | Ok(_) => std::process::exit(0),
            | Err(e) => {
                print_err(e);
                std::process::exit(1)
            }
        }
    }
}

fn print_err<E: Into<CliError>>(err: E) {
    let cli_error: CliError = err.into();
    match cli_error {
        | CliError::ClapError(e) => {
            // Clap handles colors
            e.print().unwrap();
        }
        | CliError::Abort => {
            print_formatted_error("Aborted!", "".to_owned());
        }
        | CliError::AbortMessage(e) => {
            print_formatted_error("", e.to_string());
        }
        | CliError::Other(e) => {
            print_formatted_error("Error: ", e.to_string());
        }
        | CliError::InvalidHandler(msg) => {
            print_formatted_error(
                "\n\n** Cling Handler Design Error **\n\n",
                msg,
            );
        }
    };
}

fn print_formatted_error(heading: &str, msg: String) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);

    stderr
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
        .unwrap();
    write!(&mut stderr, "{}", heading).unwrap();
    stderr.reset().unwrap();
    writeln!(&mut stderr, "{}", msg).unwrap();
}

pub(crate) fn format_clap_error<I: CommandFactory>(
    err: clap::Error,
) -> clap::Error {
    let mut cmd = I::command();
    err.format(&mut cmd)
}
