use clap::{CommandFactory, Parser};

use super::command::CliRunnable;
use crate::args::CollectedArgs;
use crate::error::CliError;
use crate::CliResult;

/// A convenience struct for parsing and running a command line interface.
///
/// Example:
/// ```
/// use cling::prelude::*;
///
/// #[derive(CliRunnable, Parser, Debug, Clone)]
/// #[cling(run = "run")]
/// pub struct MyOpts {
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
///     Cling::<MyOpts>::run().await.print_err_and_exit();
/// }
/// ```
pub struct Cling<T: CliApp + Parser> {
    _marker: std::marker::PhantomData<T>,
}
/// Parses T with clap and runs until completion
impl<T: CliApp + Parser> Cling<T> {
    pub fn parse() -> Result<T, CliError> {
        Ok(<T as clap::Parser>::try_parse().map_err(format_clap_error::<T>)?)
    }

    /// Runs the command line interface with the given state. State must
    /// implement Clone.
    pub async fn run_with_state<S>(state: S) -> CliResult
    where
        S: Clone + Send + Sync + 'static,
    {
        let parsed =
            <T as clap::Parser>::try_parse().map_err(format_clap_error::<T>)?;
        <T as CliApp>::run_with_state(&parsed, state).await
    }

    pub async fn run() -> CliResult {
        let parsed =
            <T as clap::Parser>::try_parse().map_err(format_clap_error::<T>)?;
        <T as CliApp>::run(&parsed).await
    }
}

/// Used when we have an instance of Cling after parsing.
#[async_trait::async_trait]
pub trait CliApp {
    async fn run(&self) -> CliResult;
    async fn run_with_state<S: Clone + Send + Sync + 'static>(
        &self,
        state: S,
    ) -> CliResult;
    async fn run_with_arguments(&self, args: CollectedArgs) -> CliResult;
}

#[async_trait::async_trait]
impl<T> CliApp for T
where
    T: CliRunnable + Sync + Send + Clone + 'static,
{
    async fn run(&self) -> CliResult {
        <Self as CliApp>::run_with_arguments(self, CollectedArgs::new()).await
    }

    async fn run_with_state<S>(&self, state: S) -> CliResult
    where
        S: Clone + Send + Sync + 'static,
    {
        // Put the state the state
        let mut args = CollectedArgs::new();
        args.insert(crate::extractors::State(state.clone()));
        <Self as CliApp>::run_with_arguments(self, args).await
    }

    async fn run_with_arguments(&self, mut args: CollectedArgs) -> CliResult {
        <Self as CliRunnable>::call(self, &mut args).await
    }
}

fn format_clap_error<I: CommandFactory>(err: clap::Error) -> clap::Error {
    let mut cmd = I::command();
    err.format(&mut cmd)
}
