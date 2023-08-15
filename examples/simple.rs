use cling::prelude::*;

#[derive(CliRunnable, Parser, Debug, Clone)]
#[cling(run = "run")]
pub struct App {
    #[command(flatten)]
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
pub async fn run(options: &Options) {
    println!("Opts: {options:?}");
}

#[tokio::main]
async fn main() {
    let app = App::parse();
    app.run_and_exit().await;
}
