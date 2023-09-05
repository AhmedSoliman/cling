use cling::prelude::*;
use colored::Colorize;
use static_assertions::assert_impl_all;

#[derive(Run, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[cling(run = "init")]
pub struct CliArgs {
    /// What?
    #[arg(short)]
    use_me: bool,
    #[clap(flatten)]
    // `collected` attribute allows collecting types that do not implement the
    // [Collect] trait. This is useful when you need to collect external types.
    // However, those types must implement [Clone] and cling will log a
    // warning if the same type is collected multiple times in the execution
    // path of the command (if logging is enabled).
    #[cling(collect)]
    verbosity: clap_verbosity_flag::Verbosity,
    // colors will be collected because [Colors] implement [Collect]
    #[arg(short)]
    colors: Option<Vec<Colors>>,
    #[command(flatten)]
    pub common: CommonArgs,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Collect, Debug, Clone)]
pub enum Colors {
    Red,
    Green,
    Blue,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum Commands {
    /// Calculate things
    Calculator(Calculator),
    /// What's my name?
    #[cling(run = "groot")]
    #[command(name = "whoami")]
    WhoAmI,
}

#[derive(Collect, Args, Debug, Clone)]
pub struct CommonArgs {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(Run, Collect, Parser, Debug, Clone)]
#[cling(run = "run_calc")]
pub struct Calculator {
    /// Enable color output
    #[arg(short, long, global = true)]
    pub color: bool,

    #[command(subcommand)]
    pub operation: CalcOperations,
}

#[derive(Run, Subcommand, Debug, Clone)]
pub enum CalcOperations {
    /// Add two numbers
    Add(AddArgs),
    /// Subtract two numbers
    Subtract(SubtractArgs),
}

#[derive(Run, Collect, Parser, Debug, Clone)]
#[cling(run = "run_add")]
pub struct AddArgs {
    pub num1: u64,
    pub num2: u64,
}

#[derive(Run, Collect, Parser, Debug, Clone)]
#[cling(run = "run_subtract")]
pub struct SubtractArgs {
    pub num1: u64,
    pub num2: u64,
}

#[derive(Clone, Debug)]
struct Database {
    _data: String,
}

async fn run_calc(
    State(database): State<Database>,
    calc: &Calculator,
) -> State<Database> {
    println!(
        "Database is currently {:?} but we will override that!",
        database
    );

    let database = Database {
        _data: "Loads of calculator data".to_owned(),
    };
    println!(">> Calculator: {:?}", calc);
    State(database)
}

async fn init(
    Collected(verbosity): Collected<clap_verbosity_flag::Verbosity>,
    // Can also be extracted by reference.
    // Collected(verbosity): Collected<&clap_verbosity_flag::Verbosity>,
    common: &CommonArgs,
    colors: &Option<Vec<Colors>>,
) -> State<Database> {
    println!(
        ">> Hello world! {:?}, color: {:?}, verbosity: {:?}",
        common,
        colors,
        verbosity.log_level().unwrap()
    );
    let database = Database {
        _data: "Loads of data".to_owned(),
    };
    State(database)
}

// Note that handlers can be sync as well.
fn groot() {
    println!("I'm groot!");
}

// my add handler
async fn run_add(
    // database state was created in init() by returning State() as an effect.
    State(database): State<Database>,
    calc: &Calculator,
    add: &AddArgs,
) -> Result<(), CliError> {
    println!("Database: {:?}", database);
    let output =
        format!("{} + {} = {}", add.num1, add.num2, add.num1 + add.num2);

    if calc.color {
        println!("{}", output.green());
    } else {
        println!("{output}");
    }

    Ok(())
}

// Note that this is SYNC handler.
// Fails in runtime, we expect AddArgs but this will never be collected because
// `AddArgs` is never visited while traversing the type tree during execution.
pub fn run_subtract(_calc: &Calculator, _add: &AddArgs) {
    println!("Never gets called!");
}

assert_impl_all!(CliArgs: ClapClingExt, cling::prelude::Parser, Send, Sync, Run);

#[tokio::main]
async fn main() -> ClingFinished<CliArgs> {
    env_logger::builder().init();
    Cling::parse().run().await
}
