//! The main entry point for the cling framework.
use std::marker::PhantomData;
use std::process::{ExitCode, Termination};

use clap::Parser;

use super::error::{format_clap_error, CliErrorHandler};
use crate::error::CliError;
use crate::params::CollectedArgs;

mod _private {
    pub struct Build;
    pub struct Ready;
    pub struct Finished;
}

use _private::*;

#[doc(hidden)]
pub trait Run: Send + Sync {
    fn call<'a>(&'a self, args: &'a mut CollectedArgs) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CliError>> + Send + 'a>>;
}

type ClingReady<T> = Cling<T, Ready>;
/// A completed run of a cling program.
///
/// This is typically used to introspect the result after running the cling
/// application, but since it implements [Termination] trait, it can be used as
/// a return type in `main()` directly.
///
/// ```rust, no_run
/// use cling::prelude::*;
///
/// #[derive(Run, Parser, Debug, Clone)]
/// #[cling(run = "run")]
/// pub struct App {
///     /// Turn debugging information on
///     #[arg(short, long, action = ArgAction::Count)]
///     pub debug: u8,
/// }
///
/// fn run() {
///     println!("Hello Program!");
/// }
///
/// // Note that tokio here is only used as an example, you can use any async runtime.
/// #[tokio::main]
/// async fn main() -> ClingFinished<App> {
///     Cling::parse_and_run().await
/// }
/// ```
pub type ClingFinished<T> = Cling<T, Finished>;

/// A Cling program.
///
/// Example:
/// ```
/// use cling::prelude::*;
///
/// #[derive(Run, Parser, Debug, Clone)]
/// #[cling(run = "run")]
/// pub struct App {
///     /// Turn debugging information on
///     #[arg(short, long, action = ArgAction::Count)]
///     pub debug: u8,
/// }
///
///
/// pub async fn run() {
///     println!("Hello, world!");
/// }
///
/// #[tokio::main]
/// async fn main() -> ClingFinished<App> {
///     Cling::parse_and_run().await
/// }
/// ```
pub struct Cling<T, S = Build> {
    settings: Settings,
    _status: PhantomData<S>,
    inner: ClingInner<T>,
}

/// Holds configuration for cling framework.
#[allow(dead_code)]
#[derive(Default, Clone)]
struct Settings {}

enum ClingInner<T> {
    Ready {
        parsed: T,
        collected_params: CollectedArgs,
    },
    Finished {
        result: Result<(), CliError>,
        collected_params: CollectedArgs,
        _parsed_type: PhantomData<T>,
    },
}

impl<T: Run + Parser> Cling<T, Finished> {
    /// Instantiate a successfully finished Cling application. This is useful
    /// when you want to return a successful Cling instance from `main()`
    /// directly.
    pub fn success() -> ClingFinished<T> {
        ClingFinished {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Finished {
                result: Ok(()),
                collected_params: CollectedArgs::new(),
                _parsed_type: PhantomData,
            },
        }
    }

    /// Instantiate a failed finished Cling application. This is useful
    /// when you want to wrap an Error into a Cling instance to be returned from
    /// `main()`.
    pub fn failed(e: impl Into<CliError>) -> ClingFinished<T> {
        ClingFinished {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Finished {
                result: Err(e.into()),
                collected_params: CollectedArgs::new(),
                _parsed_type: PhantomData,
            },
        }
    }
}

/// Parses T with clap and runs until completion
impl<T: Run + Parser> Cling<T, Build> {
    /// Create a Cling application from a parsed clap struct.
    pub fn new(parsed: T) -> ClingReady<T> {
        ClingReady {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed,
                collected_params: CollectedArgs::new(),
            },
        }
    }

    /// [Provisional]
    #[allow(dead_code)]
    fn with_settings(parsed: T, settings: Settings) -> ClingReady<T> {
        ClingReady {
            settings,
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed,
                collected_params: CollectedArgs::new(),
            },
        }
    }

    /// Parse command line arguments, run the program, and return the finished
    /// Cling application. [[`ClingFinished<T>`]] can be returned from `main()`
    /// directly which will handle printing errors and exiting with the
    /// correct exit code.
    pub async fn parse_and_run() -> ClingFinished<T> {
        let parsed =
            <T as clap::Parser>::try_parse().map_err(format_clap_error::<T>);
        match parsed {
            | Ok(parsed) => Cling::new(parsed).run().await,
            | Err(e) => {
                ClingFinished {
                    settings: Settings::default(),
                    _status: PhantomData,
                    inner: ClingInner::Finished {
                        result: Err(e.into()),
                        collected_params: CollectedArgs::new(),
                        _parsed_type: PhantomData,
                    },
                }
            }
        }
    }

    /// Parse command line arguments and exit if parsing failed.
    pub fn parse() -> ClingReady<T> {
        ClingReady {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed: <T as clap::Parser>::parse(),
                collected_params: CollectedArgs::new(),
            },
        }
    }

    /// Attempt to parse command line arguments and return a runnable Cling
    /// application.
    pub fn try_parse() -> Result<ClingReady<T>, CliError> {
        Ok(ClingReady {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed: <T as clap::Parser>::try_parse()
                    .map_err(format_clap_error::<T>)?,
                collected_params: CollectedArgs::new(),
            },
        })
    }

    pub fn try_parse_from<I, B>(itr: I) -> Result<ClingReady<T>, CliError>
    where
        I: IntoIterator<Item = B>,
        B: Into<std::ffi::OsString> + Clone,
    {
        Ok(ClingReady {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed: <T as clap::Parser>::try_parse_from(itr)
                    .map_err(format_clap_error::<T>)?,
                collected_params: CollectedArgs::new(),
            },
        })
    }

    /// Parses input as a UNIX shell command.
    ///
    /// Example input string: `sub-command --debug=2`. Note that the input
    /// **must omit** the CLI binary name, otherwise clap parsing will
    /// fail.
    #[cfg(feature = "shlex")]
    pub fn try_parse_str(input: &str) -> Result<ClingReady<T>, CliError> {
        // binary name
        let bin_name = std::env::current_exe()
            .ok()
            .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
            .and_then(|s| s.into_string().ok())
            .unwrap();
        Self::try_parse_str_with_bin_name(&bin_name, input)
    }

    #[cfg(feature = "shlex")]
    pub fn try_parse_str_with_bin_name(
        bin_name: &str,
        input: &str,
    ) -> Result<ClingReady<T>, CliError> {
        let input = format!("{bin_name} {input}");
        let args = shlex::split(&input).ok_or(CliError::InputString)?;
        let parsed = <T as clap::Parser>::try_parse_from(args)
            .map_err(format_clap_error::<T>)?;
        Ok(ClingReady {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed,
                collected_params: CollectedArgs::new(),
            },
        })
    }

    /// Parse command line arguments and aborts the program if parsing failed.
    pub fn parse_or_exit() -> ClingReady<T> {
        ClingReady {
            settings: Settings::default(),
            _status: PhantomData,
            inner: ClingInner::Ready {
                parsed: <T as clap::Parser>::try_parse()
                    .map_err(format_clap_error::<T>)
                    .unwrap_or_exit(),
                collected_params: CollectedArgs::new(),
            },
        }
    }

    /// Parses command line arguments, runs the program and exits afterwards.
    pub async fn default_run_and_exit() -> ! {
        Self::parse_or_exit().run_and_exit().await
    }
}

/// Cling is now ready to run.
impl<T: Run + Parser> Cling<T, Ready> {
    pub async fn run_and_exit(self) -> ! {
        let res = self.run().await;
        res.result().then_exit()
    }

    pub async fn run_with_state_and_exit<S>(self, state: S) -> !
    where
        S: Clone + Send + Sync + 'static,
    {
        self.run_with_state(state).await.result().then_exit()
    }

    /// Runs the app with a given state.
    pub async fn run(self) -> ClingFinished<T> {
        let ClingInner::Ready {
            parsed,
            mut collected_params,
        } = self.inner
        else {
            // This will never happen. run() is only implemented on
            // Cling::Ready.
            unreachable!()
        };

        let result = <T as Run>::call(&parsed, &mut collected_params).await;
        // We ensure that transitioning to ClingFinished only happens when we
        // have a result. Therefore, it's safe to unwrap() the result in
        // ClingFinished.
        ClingFinished {
            settings: self.settings,
            _status: PhantomData,
            inner: ClingInner::Finished {
                collected_params,
                result,
                _parsed_type: PhantomData,
            },
        }
    }

    pub async fn run_with_state<S>(mut self, state: S) -> ClingFinished<T>
    where
        S: Clone + Send + Sync + 'static,
    {
        let ClingInner::Ready {
            ref mut collected_params,
            ..
        } = self.inner
        else {
            // This will never happen. run_with_state() is only implemented on
            // Cling::Ready.
            unreachable!()
        };
        // Put the state the state
        collected_params.insert(
            crate::extractors::State(state),
            /* override_is_expected = */ true,
        );
        Self::run(self).await
    }
}

/// Cling program has terminated and results can be introspected.
impl<T: Run + Parser> Cling<T, Finished> {
    pub fn result_ref(&self) -> &Result<(), CliError> {
        let ClingInner::Finished { ref result, .. } = self.inner else {
            unreachable!()
        };
        result
    }

    pub fn result(self) -> Result<(), CliError> {
        let ClingInner::Finished { result, .. } = self.inner else {
            unreachable!()
        };
        result
    }

    pub fn is_success(&self) -> bool {
        self.result_ref().is_ok()
    }

    pub fn is_failure(&self) -> bool {
        self.result_ref().is_err()
    }

    pub fn collected_parameters(&self) -> &CollectedArgs {
        let ClingInner::Finished {
            ref collected_params,
            ..
        } = self.inner
        else {
            unreachable!()
        };
        collected_params
    }

    pub fn collected_arguments_mut(&mut self) -> &mut CollectedArgs {
        let ClingInner::Finished {
            ref mut collected_params,
            ..
        } = self.inner
        else {
            unreachable!()
        };
        collected_params
    }
}

/// Enables clap structs to be executed with cling.
///
/// This extension trait allows clap users to parse their clap structs as usual,
/// then run them with cling without constructing a [Cling] instance
/// explicitly.
///
/// Example:
/// ```
/// use cling::prelude::*;
///
/// #[derive(Run, Parser, Debug, Clone)]
/// #[cling(run = "run")]
/// pub struct App {
///     /// Turn debugging information on
///     #[arg(short, long, action = ArgAction::Count)]
///     pub debug: u8,
/// }
///
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
pub trait ClapClingExt: Sized {
    fn into_cling(self) -> ClingReady<Self>;
}

impl<T> ClapClingExt for T
where
    T: Run + Parser + Sync + Send + 'static,
{
    fn into_cling(self) -> ClingReady<Self> {
        Cling::<T>::new(self)
    }
}

/// Allows main() to return ClingFinished and it'll report the error correctly
/// if any.
impl<T: Run + Parser> Termination for ClingFinished<T> {
    fn report(self) -> ExitCode {
        if let Err(e) = self.result() {
            // Silently ignore IO errors.
            let _ = e.print();
            return ExitCode::from(e.exit_code());
        }
        ExitCode::SUCCESS
    }
}

/// Convert a [CliError] into a [`ClingFinished`].
impl<T: Run + Parser> From<CliError> for ClingFinished<T> {
    fn from(value: CliError) -> Self {
        Cling::failed(value)
    }
}
