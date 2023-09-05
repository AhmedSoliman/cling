use cling::_private::{
    Collect,
    CollectedArgs,
    HandlerEffect,
    IntoEffect,
    SetState,
};
use cling::prelude::*;

#[derive(Clone, Debug)]
struct CommonOpts;

#[derive(Clone, Debug)]
struct NotSoCommonOpts;

impl Collect for CommonOpts {}

impl Collect for NotSoCommonOpts {}

async fn noop(
    opts: &CommonOpts,
    other_opts: &NotSoCommonOpts,
) -> Result<(), anyhow::Error> {
    println!("async noop: {:?} {:?}", opts, other_opts);
    Ok(())
}

fn sync_noop(
    opts: &CommonOpts,
    other_opts: &NotSoCommonOpts,
) -> Result<(), anyhow::Error> {
    println!("sync noop: {:?} {:?}", opts, other_opts);
    Ok(())
}

// Handlers can be unit
fn unit_handler(_opts: &CommonOpts, _other_opts: &NotSoCommonOpts) {
    println!("do nothing");
}

// Can return result of CliError
fn handler_fails(
    _opts: &CommonOpts,
    _other_opts: &NotSoCommonOpts,
) -> Result<(), CliError> {
    Err(CliError::FailedWithMessage("handler failed".to_owned()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Database;

// Can return state effect in result
fn handler_with_effect_result(
    _opts: &CommonOpts,
    _other_opts: &NotSoCommonOpts,
) -> Result<State<Database>, CliError> {
    Ok(State(Database))
}

// Can return state extract as an effect
fn handler_with_state_extractor_effect(
    _opts: &CommonOpts,
    _other_opts: &NotSoCommonOpts,
) -> State<Database> {
    State(Database)
}

// Can return state effect directly
fn handler_with_effect(
    _opts: &CommonOpts,
    _other_opts: &NotSoCommonOpts,
) -> SetState<Database> {
    SetState(Database)
}

async fn handle<
    'a,
    Type,
    Input,
    Effect: HandlerEffect,
    Output: IntoEffect<Type, Effect = Effect>,
    T: Handler<'a, Type, Input, Output, Effect>,
>(
    args: &'a mut CollectedArgs,
    handler: T,
) -> Result<Effect, CliError> {
    handler.call(args)?.into_effect().await
}

#[tokio::test]
async fn handler_tests() -> Result<(), CliError> {
    let mut args = CollectedArgs::default();
    args.insert(CommonOpts, false);
    args.insert(NotSoCommonOpts, false);

    assert_eq!((), handle(&mut args, noop).await?);
    assert_eq!((), handle(&mut args, sync_noop).await?);
    assert_eq!((), handle(&mut args, unit_handler).await?);
    assert_eq!((), handle(&mut args, unit_handler).await?);
    assert!(handle(&mut args, handler_fails).await.is_err());

    assert_eq!(
        SetState(Database),
        handle(&mut args, handler_with_effect_result).await?
    );

    assert_eq!(
        SetState(Database),
        handle(&mut args, handler_with_state_extractor_effect).await?
    );

    assert_eq!(
        SetState(Database),
        handle(&mut args, handler_with_effect).await?
    );

    Ok(())
}
