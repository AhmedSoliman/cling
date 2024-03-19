use cling::prelude::*;

#[derive(Run, Parser, Debug, Clone)]
#[cling(run = "run")]
pub struct App {}

// handlers can be sync or async, cling will handle this transparently.
async fn run() -> anyhow::Result<()> {
    let err1 = std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        "Fatal disk IO Error!",
    );
    let err2 = anyhow::Error::new(err1).context("Trying to read a file");
    let err3 = err2.context("Can't load application");
    let err4 = err3.context("App level error");
    Err(err4)
}

#[tokio::main]
async fn main() -> ClingFinished<App> {
    env_logger::builder().init();
    // Cling::parse().run().await
    // Or, return ClingFinished<T> to let cling handle error printing and exit
    // code in a more convenient way.
    Cling::parse_and_run().await
}
