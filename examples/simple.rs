use cling::prelude::*;
use colored::Colorize;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliOpts {
    #[command(flatten)]
    pub common: CommonOpts,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Hey this is the first command
    Calculator(CalculatorOpts),
    /// Hey this is the second command
    WhoAmI,
}

#[derive(Parser, Debug, Clone)]
pub struct CommonOpts {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(Parser, Debug, Clone)]
pub struct CalculatorOpts {
    /// Enable color output
    #[arg(short, long, global = true)]
    pub color: bool,

    #[command(subcommand)]
    pub operation: CalcOperations,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CalcOperations {
    /// Add two numbers
    Add(AddOpts),
    /// Subtract two numbers
    Subtract(SubtractOpts),
}

#[derive(Parser, Debug, Clone)]
pub struct AddOpts {
    pub num1: u64,
    pub num2: u64,
}

#[derive(Parser, Debug, Clone)]
pub struct SubtractOpts {
    pub num1: u64,
    pub num2: u64,
}

// my add handler
pub fn run_add(
    calc_opts: &CalculatorOpts,
    add_opts: &AddOpts,
) -> Result<(), CliError> {
    let output = format!(
        "{} + {} = {}",
        add_opts.num1,
        add_opts.num2,
        add_opts.num1 + add_opts.num2
    );
    if calc_opts.color {
        println!("{}", output.green());
    } else {
        println!("{output}");
    }

    Ok(())
}

pub fn noop() {}

fn main() -> CliResult {
    let _opts = CliOpts::parse();
    // match &opts.command {
    //     | Commands::Calculator(calc_opts) => {
    //         match &calc_opts.operation {
    //             | CalcOperations::Add(add_opts) => run_add(calc_opts,
    // add_opts),             | CalcOperations::Subtract(sub_opts) =>
    // noop(),         }
    //     }
    //     | Commands::WhoAmI => noop(),
    // }
    Ok(())
}
