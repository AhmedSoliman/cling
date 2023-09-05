use cling::prelude::*;

// -- args --

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, long_about = None)]
/// A simple multi-command example using cling
struct MyApp {
    #[clap(flatten)]
    pub opts: CommonOpts,
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Collect, Parser, Debug, Clone)]
struct CommonOpts {
    /// Turn debugging information on
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

// Commands for the app are defined here.
#[derive(Run, Subcommand, Debug, Clone)]
enum Command {
    /// Honk the horn!
    Honk(HonkOpts),
    #[cling(run = "beep")]
    /// Beep beep!
    Beep,
}

// Options of "honk" command. We define cling(run=...) here to call the
// function when this command is executed.
#[derive(Run, Collect, Parser, Debug, Clone)]
#[cling(run = "honk")]
struct HonkOpts {
    /// How many times to honk?
    times: u8,
}

// -- Handlers --

// We can access &CommonOpts because it derives [Collect]
fn honk(common_opts: &CommonOpts, HonkOpts { times }: &HonkOpts) {
    if common_opts.verbose > 0 {
        println!("Honking {} times", times);
    }

    (0..*times).for_each(|_| {
        print!("Honk ");
    });
    println!("!");
}

// Maybe beeps need to be async!
async fn beep() {
    println!("Beep, Beep!");
}

// -- main --

#[tokio::main]
async fn main() -> ClingFinished<MyApp> {
    env_logger::builder().init();
    Cling::parse_and_run().await
}
