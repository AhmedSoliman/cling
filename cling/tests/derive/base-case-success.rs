use cling::prelude::*;

#[derive(Run, Parser, Debug, Clone)]
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
    let options = Options::parse();
    options.into_cling().run_and_exit().await;
}
