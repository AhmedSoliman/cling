use cling::prelude::*;
use colored::Colorize;
use static_assertions::assert_impl_all;

#[derive(CliRunnable, CliParam, Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[cling(run = "init")]
pub struct CliOpts {
    /// What?
    #[arg(short)]
    use_me: bool,
    #[clap(flatten)]
    // `collected` attribute allows collecting types that do not implement the
    // CliParam trait. This is useful when you need to collect external types.
    // However, those types will need to implement `Clone` and cling will log a
    // warning if the same type is collected multiple times in the execution
    // path of the command.
    #[cling(collect)]
    verbosity: clap_verbosity_flag::Verbosity,
    #[arg(short)]
    colors: Option<Vec<Colors>>,
    #[command(flatten)]
    pub common: CommonOpts,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(CliParam, ValueEnum, Debug, Clone)]
pub enum Colors {
    Red,
    Green,
    Blue,
}

#[derive(CliRunnable, Subcommand, Debug, Clone)]
pub enum Commands {
    /// Calculate things
    Calculator(Calculator),
    /// What's my name?
    #[cling(run = "groot")]
    #[command(name = "whoami")]
    WhoAmI,
}

#[derive(CliParam, Parser, Debug, Clone)]
pub struct CommonOpts {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,
}

#[derive(CliRunnable, CliParam, Parser, Debug, Clone)]
#[cling(run = "run_calc")]
pub struct Calculator {
    /// Enable color output
    #[arg(short, long, global = true)]
    pub color: bool,

    #[command(subcommand)]
    pub operation: CalcOperations,
}

#[derive(CliRunnable, Subcommand, Debug, Clone)]
pub enum CalcOperations {
    /// Add two numbers
    Add(AddOpts),
    /// Subtract two numbers
    Subtract(SubtractOpts),
}

#[derive(CliRunnable, CliParam, Parser, Debug, Clone)]
#[cling(run = "run_add")]
pub struct AddOpts {
    pub num1: u64,
    pub num2: u64,
}

#[derive(CliRunnable, Parser, Debug, Clone)]
#[cling(run = "run_subtract")]
pub struct SubtractOpts {
    pub num1: u64,
    pub num2: u64,
}

#[derive(Clone, Debug)]
struct Database {
    _data: String,
}

async fn run_calc(calc: &Calculator) {
    println!(">> Calculator: {:?}", calc);
}

async fn init(
    State(database): State<Database>,
    Collected(verbosity): Collected<clap_verbosity_flag::Verbosity>,
    // Can also be extracted by reference.
    // Collected(verbosity): Collected<&clap_verbosity_flag::Verbosity>,
    common: &CommonOpts,
    colors: Option<Vec<Colors>>,
) {
    println!(
        ">> Hello world! {:?}, color: {:?}, database: {:?}, verbosity: {:?}",
        common,
        colors,
        database,
        verbosity.log_level().unwrap()
    );
}

// Note that handlers can be sync as well.
fn groot() {
    println!("I'm groot!");
}

// my add handler
pub async fn run_add(
    calc: &Calculator,
    add_opts: &AddOpts,
) -> Result<(), CliError> {
    let output = format!(
        "{} + {} = {}",
        add_opts.num1,
        add_opts.num2,
        add_opts.num1 + add_opts.num2
    );

    if calc.color {
        println!("{}", output.green());
    } else {
        println!("{output}");
    }

    Ok(())
}

// Note that this is SYNC handler.
// Fails in runtime, we expect AddOpts but this will never be collected in the
// subtraction path.
pub fn run_subtract(_calc: &Calculator, _add_opts: &AddOpts) {
    println!("Never gets called!");
}

assert_impl_all!(CliOpts: ClapClingExt, cling::prelude::Parser, Send, Sync, CliRunnable);

#[tokio::main]
async fn main() -> ClingFinished<CliOpts> {
    env_logger::builder().init();
    let database = Database {
        _data: "Loads of data".to_owned(),
    };
    Cling::parse().run_with_state(database).await
}
