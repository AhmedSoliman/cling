use anyhow::bail;
use cling::prelude::*;

#[derive(CliRunnable, CliParam, Parser, Debug, Clone)]
#[cling(run = "run")]
pub struct App {
    #[clap(flatten)]
    pub options: Options,
}

// Structs that derive CliParam are optionally available for handlers as
// parameters both as value and reference.
#[derive(CliParam, Parser, Debug, Clone)]
pub struct Options {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

// handlers can be sync or async, cling will handle this transparently.
async fn run(options: &Options) -> Result<(), anyhow::Error> {
    println!("Opts: {options:?}");
    if options.debug > 3 {
        bail!("Too much debugging");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ClingFinished<App> {
    // Cling::parse().run().await
    // Or, return ClingFinished<T> to let cling handle error printing and exit
    // code in a more convenient way.
    Cling::parse_and_run().await
}
