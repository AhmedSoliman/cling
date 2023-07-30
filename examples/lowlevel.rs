use cling::args::{CliParam, CollectedArgs};
use cling::prelude::*;

#[derive(Clone, Debug)]
struct CommonOpts;

#[derive(Clone, Debug)]
struct NotSoCommonOpts;

impl<'a> CliParam<'a> for CommonOpts {
    fn from_args(args: &'a CollectedArgs) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}

impl<'a> CliParam<'a> for NotSoCommonOpts {
    fn from_args(args: &'a CollectedArgs) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}

async fn noop(
    // by value,
    opts: CommonOpts,
    // see, we can also take reference!
    other_opts: &NotSoCommonOpts,
) -> Result<(), anyhow::Error> {
    println!("noop: {:?} {:?}", opts, other_opts);
    Ok(())
}

async fn handle<'a, X, T: CliHandler<'a, X>>(
    args: &'a mut CollectedArgs,
    handler: T,
) -> CliResult {
    handler.call(args).await
}

#[tokio::main]
async fn main() -> CliResult {
    let mut args = CollectedArgs::default();
    args.insert(CommonOpts);
    args.insert(NotSoCommonOpts);

    handle(&mut args, noop).await
}
