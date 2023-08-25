use cling::_private::{CliParam, CollectedParams, IntoCliResult};
use cling::prelude::*;

#[derive(Clone, Debug)]
struct CommonOpts;

#[derive(Clone, Debug)]
struct NotSoCommonOpts;

impl<'a> CliParam<'a> for CommonOpts {
    fn extract_param(args: &'a CollectedParams) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}

impl<'a> CliParam<'a> for NotSoCommonOpts {
    fn extract_param(args: &'a CollectedParams) -> Option<Self> {
        args.get::<Self>().cloned()
    }
}

async fn noop(
    // by value,
    opts: CommonOpts,
    // see, we can also take reference!
    other_opts: &NotSoCommonOpts,
) -> Result<(), anyhow::Error> {
    println!("async noop: {:?} {:?}", opts, other_opts);
    Ok(())
}

fn sync_noop(
    // by value,
    opts: CommonOpts,
    // see, we can also take reference!
    other_opts: &NotSoCommonOpts,
) -> Result<(), anyhow::Error> {
    println!("sync noop: {:?} {:?}", opts, other_opts);
    Ok(())
}

async fn handle<
    'a,
    Type,
    Output: IntoCliResult<Type>,
    X,
    T: CliHandler<'a, Type, X, Output>,
>(
    args: &'a mut CollectedParams,
    handler: T,
) -> Result<(), CliError> {
    handler.call(args)?.into_result().await
}

#[tokio::test]
async fn handler_tests() -> Result<(), CliError> {
    let mut args = CollectedParams::default();
    args.insert(CommonOpts);
    args.insert(NotSoCommonOpts);

    handle(&mut args, noop).await?;
    handle(&mut args, sync_noop).await?;
    Ok(())
}
