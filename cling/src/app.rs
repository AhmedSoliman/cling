//! The main entry point for the cling framework.
use std::marker::PhantomData;

use clap::Parser;

use super::command::CliRunnable;
use super::error::{format_clap_error, CliErrorHandler};
use crate::args::CollectedArgs;
use crate::error::CliError;
use crate::CliResult;

pub struct New;
pub struct ReadyToRun;
pub struct Finished;

pub type ClingNew<T> = Cling<T, New>;
pub type ClingReady<T> = Cling<T, ReadyToRun>;
pub type ClingFinished<T> = Cling<T, Finished>;
/// A convenience struct for parsing and running a command line interface.
///
/// Example:
/// ```
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
///
/// pub async fn run() {
///     println!("Hello, world!");
/// }
///
/// #[tokio::main]
/// async fn main() {
///     Cling::<App>::default_run_and_exit()
///       .await;
/// }
/// ```
pub struct Cling<T, S = New> {
    parsed: T,
    collected_args: CollectedArgs,
    result: Option<CliResult>,
    _status: PhantomData<S>,
}
/// Parses T with clap and runs until completion
impl<T: CliRunnable + Parser> Cling<T, New> {
    /// Create a Cling application from a parsed clap struct.
    pub fn new(parsed: T) -> Self {
        Self {
            parsed,
            collected_args: CollectedArgs::new(),
            result: None,
            _status: PhantomData,
        }
    }

    /// Attempt to parse command line arguments and return a runnable Cling
    /// application.
    pub fn try_parse() -> Result<ClingReady<T>, CliError> {
        Ok(Cling {
            parsed: <T as clap::Parser>::try_parse()
                .map_err(format_clap_error::<T>)?,
            collected_args: CollectedArgs::new(),
            result: None,
            _status: PhantomData,
        })
    }

    /// Parse command line arguments and aborts the program if parsing failed.
    pub fn parse_or_exit() -> ClingReady<T> {
        Cling {
            parsed: <T as clap::Parser>::try_parse()
                .map_err(format_clap_error::<T>)
                .unwrap_or_exit(),
            collected_args: CollectedArgs::new(),
            result: None,
            _status: PhantomData,
        }
    }

    /// Parses command line arguments, runs the program and exits afterwards.
    pub async fn default_run_and_exit() -> ! {
        Self::parse_or_exit().run_and_exit().await
    }
}

/// Cling is now ready to run.
impl<T: CliRunnable + Parser> Cling<T, ReadyToRun> {
    pub async fn run_and_exit(self) -> ! {
        self.run().await.result().then_exit()
    }

    pub async fn run_with_state_and_exit<S>(self, state: S) -> !
    where
        S: Clone + Send + Sync + 'static,
    {
        self.run_with_state(state).await.result().then_exit()
    }

    /// Runs the app with a given state.
    pub async fn run(mut self) -> ClingFinished<T> {
        let result =
            <T as CliRunnable>::call(&self.parsed, &mut self.collected_args)
                .await;
        // We ensure that transitioning to ClingFinished only happens when we
        // have a result. Therefore, it's safe to unwrap() the result in
        // ClingFinished.
        ClingFinished {
            parsed: self.parsed,
            collected_args: self.collected_args,
            result: Some(result),
            _status: PhantomData,
        }
    }

    pub async fn run_with_state<S>(mut self, state: S) -> ClingFinished<T>
    where
        S: Clone + Send + Sync + 'static,
    {
        // Put the state the state
        self.collected_args.insert(crate::extractors::State(state));
        Self::run(self).await
    }
}

/// Cling program has terminated and results can be introspected.
impl<T: CliRunnable + Parser> Cling<T, Finished> {
    pub fn result(self) -> CliResult {
        self.result.unwrap()
    }

    pub fn is_successful(&self) -> bool {
        self.result.as_ref().unwrap().is_ok()
    }

    pub fn collected_arguments(&self) -> &CollectedArgs {
        &self.collected_args
    }

    pub fn collected_arguments_mut(&mut self) -> &mut CollectedArgs {
        &mut self.collected_args
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
/// #[derive(CliRunnable, Parser, Debug, Clone)]
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
///     app.run_and_exit().await;
/// }
/// ```
#[async_trait::async_trait]
pub trait ClapClingExt {
    async fn run(&self) -> CliResult;
    async fn run_and_exit(self);
    async fn run_with_state<S: Clone + Send + Sync + 'static>(
        &self,
        state: S,
    ) -> CliResult;
    async fn run_with_state_and_exit<S: Clone + Send + Sync + 'static>(
        self,
        state: S,
    );
}

#[async_trait::async_trait]
impl<T> ClapClingExt for T
where
    T: CliRunnable + Parser + Sync + Send + Clone + 'static,
{
    async fn run(&self) -> CliResult {
        let app = Cling::<T>::try_parse()?;
        app.run().await.result()
    }

    async fn run_and_exit(self) {
        let app = Cling::<T>::try_parse().unwrap_or_exit();
        app.run_and_exit().await;
    }

    async fn run_with_state<S: Clone + Send + Sync + 'static>(
        &self,
        state: S,
    ) -> CliResult {
        let app = Cling::<T>::try_parse()?;
        app.run_with_state(state).await.result()
    }

    async fn run_with_state_and_exit<S: Clone + Send + Sync + 'static>(
        self,
        state: S,
    ) {
        let app = Cling::<T>::try_parse().unwrap_or_exit();
        app.run_with_state_and_exit(state).await;
    }
}
