use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use thiserror::Error;

pub trait CliErrorHandler {
    fn print_err_and_exit(self);
    fn print_err(self);
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid Handler: {0}")]
    InvalidHandler(String),
    #[error("Internal Error: {0}")]
    InternalError(String),
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
    fn print_err_and_exit(self) {
        match self {
            | Ok(_) => {}
            | Err(_) => {
                self.print_err();
                std::process::exit(1);
            }
        };
    }

    fn print_err(self) {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);

        match self {
            | Ok(_) => {}
            | Err(e) => {
                let cli_error: CliError = e.into();
                match cli_error {
                    | CliError::ClapError(e) => {
                        // Clap handles colors
                        e.print().unwrap();
                    }
                    | CliError::InternalError(_) => todo!(),
                    | CliError::Abort => todo!(),
                    | CliError::AbortMessage(_) => todo!(),
                    | CliError::Other(_) => todo!(),
                    | CliError::InvalidHandler(msg) => {
                        stderr
                            .set_color(
                                ColorSpec::new()
                                    .set_fg(Some(Color::Red))
                                    .set_bold(true),
                            )
                            .unwrap();
                        write!(
                            &mut stderr,
                            "\n\n** Cling Handler Design Error **\n\n"
                        )
                        .unwrap();
                        stderr.reset().unwrap();
                        write!(&mut stderr, "{}", msg).unwrap();
                    }
                };
            }
        };
        stderr.reset().unwrap();
    }
}
