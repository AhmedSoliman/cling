use cling::prelude::*;

#[derive(Run, Parser, Debug, Clone)]
#[cling(run = "run")]
pub struct App {
    /// Tell me your name
    #[arg(long)]
    #[cling(collect)]
    pub name: String,

    /// Your city name
    #[arg(long)]
    #[cling(collect)]
    pub city: String,
}

fn run(
    Collected(name): Collected<String>,
    Collected(city): Collected<String>,
) -> Result<(), anyhow::Error> {
    println!("Your name is: {name} and you are in {city}!");
    println!(
        "This is definitely the wrong result. This is because we \
         `#[cling(collect)]`ed the same type 'String' twice, the last \
         collected will always win. \n\nRun with `RUST_LOG=warn` to see the \
         warning the cling generates, or `RUST_LOG=trace` to see details when \
         arguments were collected."
    );
    Ok(())
}

#[tokio::main]
async fn main() -> ClingFinished<App> {
    env_logger::init();
    Cling::parse_and_run().await
}
