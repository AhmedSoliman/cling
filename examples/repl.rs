use std::io::Write;

use cling::prelude::*;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[cling(run = "init")]
pub struct CliOpts {
    #[clap(flatten)]
    pub common: CommonOpts,
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum Commands {
    /// Ask me a question
    #[command(name = "ask")]
    #[cling(run = "ask")]
    AskQuestion,
    // Guess a number between 1 and 10
    Guess(Guess),
    /// Who am I?
    #[command(name = "whoami")]
    #[cling(run = "whoami")]
    WhoAmI,
}

#[derive(Collect, Args, Debug, Clone)]
pub struct CommonOpts {
    /// Turn debugging information on
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Run, Args, Collect, Debug, Clone)]
#[cling(run = "guess")]
pub struct Guess {
    pub num: u8,
}

// Handlers
fn ask() {
    let mut name = String::new();
    print!("What is your name? ");
    std::io::stdout().flush().unwrap();

    std::io::stdin().read_line(&mut name).unwrap();
    println!(">> Hello {}", name);
}

async fn init(common: &CommonOpts) {
    if common.verbose > 0 {
        println!("Initializing...");
    }
}

fn whoami() {
    println!("I'm groot!");
}

pub fn guess(guess: &Guess) -> Result<(), CliError> {
    let random = rand::random::<u8>() % 10;
    if guess.num == random {
        println!("{}", "You guessed it right!".green());
    } else {
        println!(
            "{} Correct answer is {}",
            "You guessed it wrong!".red(),
            random
        );
        return Err(CliError::Failed);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> ClingFinished<CliOpts> {
    // Setting up a simple REPL loop.
    println!("Welcome to the sample REPL!");
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            | Ok(line) => {
                let opts = Cling::<CliOpts>::try_parse_str(&line);
                match opts {
                    | Ok(opts) => {
                        let output = opts.run().await;
                        if let Err(e) = output.result() {
                            e.print().unwrap();
                        }
                    }
                    | Err(e) => e.print().unwrap(),
                };
            }
            | Err(ReadlineError::Interrupted) => {
                // Program terminates with exit code 1
                return Cling::failed(CliError::FailedWithMessage(
                    "CTRL-C".to_owned(),
                ));
            }
            | Err(ReadlineError::Eof) => {
                // Program terminates with exit code 0
                println!("CTRL-D");
                return Cling::success();
            }
            | Err(err) => {
                return Cling::failed(CliError::Other(err.into()));
            }
        }
    }
}
