use cling::prelude::*;

#[derive(CliRunnable, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[cling(run = "run")]
pub struct Options {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

pub async fn run() {
    println!("Hello, world!");
}

#[tokio::main]
async fn main() {
    Cling::<Options>::run().await.print_err_and_exit();
    // or
    let options = Options::parse();
    let _ = options.run().await.print_err_and_exit();
}
