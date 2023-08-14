use cling::prelude::*;

#[derive(CliRunnable, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct MyApp {
    #[clap(flatten)]
    pub opts: CommonOpts,
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(CliParam, Parser, Debug, Clone)]
struct CommonOpts {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

// Commands for the app are defined here.
#[derive(CliRunnable, Subcommand, Debug, Clone)]
enum Command {
    Honk(HonkOpts),
    #[cling(run = "beep")]
    Beep,
}

// Options of "honk" command. We define cling(run=...) here to call the
// function when this command is executed.
#[derive(CliRunnable, CliParam, Parser, Debug, Clone)]
#[cling(run = "honk")]
struct HonkOpts {
    /// How many times to honk?
    times: u8,
}

fn honk(common_opts: &CommonOpts, HonkOpts { times }: HonkOpts) {
    if common_opts.verbose > 0 {
        println!("Honking {} times", times);
    }

    for _ in 0..times {
        print!("Honk ");
    }
    println!("!");
}

fn beep() {
    println!("Beep, Beep!");
}

#[tokio::main]
async fn main() {
    Cling::<MyApp>::run().await.print_err_and_exit();
}
