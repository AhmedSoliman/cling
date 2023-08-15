use crate::args::CollectedArgs;
use crate::prelude::CliError;

/// Marks clap structs as cling runnable.
///
/// This trait needs to be derived for clap structs or enums that will run a
/// function handler or a if has subcommands. Note that cling requires that all
/// clap structs/enums implement [Clone].
///
/// Usually, this will be derived like the following example:
/// ```
/// 
/// use cling::prelude::*;
///
/// #[derive(CliRunnable, Parser, Debug, Clone)]
/// #[cling(run = "do_nothing")]
/// pub struct App {
///     /// Turn debugging information on
///     #[arg(short, long, action = ArgAction::Count)]
///     pub debug: u8,
/// }
///
/// fn do_nothing() {}
/// ```
///
/// Runnable structs will execute the handler specified in `#[cling(run =
/// "...")]` The string value must be a valid path to a function. See details
/// about how to write cling handlers in [super::handler] module.
#[async_trait::async_trait]
pub trait CliRunnable: Send + Sync {
    async fn call(&self, args: &mut CollectedArgs) -> Result<(), CliError>;
}
