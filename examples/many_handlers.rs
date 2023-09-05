use cling::prelude::*;

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[cling(run = "init")]
pub struct MyApp {
    #[clap(flatten)]
    pub common: CommonOpts,
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Args, Collect, Debug, Clone)]
pub struct CommonOpts {
    /// Access token
    #[arg(long, global = true)]
    pub access_token: Option<String>,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum Commands {
    #[cling(run = "run_beep")]
    Beep,
    /// Self identification
    #[cling(run = "run_whoami")]
    WhoAmI,
}

fn init() {
    println!("init handler!");
}

fn run_whoami() {
    println!("I'm groot!");
}

fn run_beep() {
    println!("Beep beep!");
}

#[tokio::main]
async fn main() -> ClingFinished<MyApp> {
    Cling::parse_and_run().await
}
