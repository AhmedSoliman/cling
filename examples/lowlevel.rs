use cling::args::{CollectedArgs, FromCliArgs};
use cling::prelude::*;

#[derive(Clone, Debug)]
struct CommonOpts;

#[derive(Clone, Debug)]
struct NotSoCommonOpts;

impl<'a> FromCliArgs<'a> for CommonOpts {
    fn from_args(args: &'a CollectedArgs) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}

impl<'a> FromCliArgs<'a> for NotSoCommonOpts {
    fn from_args(args: &'a CollectedArgs) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}

fn noop(
    opts: CommonOpts,
    other_opts: NotSoCommonOpts,
) -> Result<(), anyhow::Error> {
    println!("noop: {:?} {:?}", opts, other_opts);
    Ok(())
}

fn handle<'a, X, T: CliHandler<'a, X>>(
    args: &'a mut CollectedArgs,
    handler: T,
) -> CliResult {
    handler.call(args)
}

fn main() -> CliResult {
    let mut args = CollectedArgs::default();
    args.insert(CommonOpts);
    args.insert(NotSoCommonOpts);

    handle(&mut args, noop)
}
