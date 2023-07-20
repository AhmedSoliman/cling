use std::any::type_name;

use crate::args::{CollectedArgs, FromCliArgs};
use crate::prelude::CliError;

/// trait for functions that can be used to handle command line commands.
pub trait CliHandler<'a, Input> {
    fn call(self, args: &'a mut CollectedArgs) -> Result<(), CliError>;
}

/// trait to handle function return types:
/// - Result<(), E> where E: Into<CliError>
/// - ()
pub trait IntoCliResult {
    fn into_result(self) -> Result<(), CliError>;
}

impl IntoCliResult for () {
    fn into_result(self) -> Result<(), CliError> {
        Ok(())
    }
}

impl<E> IntoCliResult for Result<(), E>
where
    E: Into<CliError>,
{
    fn into_result(self) -> Result<(), CliError> {
        self.map_err(Into::into)
    }
}

// we want to handle functions that return:
// 1. Result<(), E> where E: Into<CliError>
// 2. ()
//
// So type that implements trait IntoCliResult is accepted.
impl<'a, F, Output> CliHandler<'a, ((),)> for F
where
    F: FnOnce() -> Output,
    Output: IntoCliResult,
{
    fn call(self, _args: &'a mut CollectedArgs) -> Result<(), CliError> {
        self().into_result()
    }
}

// we want to handle functions that return:
// 1. Result<(), E> where E: Into<CliError>
// 2. ()
#[allow(non_snake_case, unused_mut)]
impl<'a, F, Output, I> CliHandler<'a, ((I,),)> for F
where
    F: FnOnce(I) -> Output,
    Output: IntoCliResult,
    I: FromCliArgs<'a>,
{
    fn call(self, args: &'a mut CollectedArgs) -> Result<(), CliError> {
        let type_1_name = type_name::<I>();
        let Some(I) = I::from_args(args) else {
            return Err(CliError::InternalError(format!(
                "Type {type_1_name} couldn't be collected from the input \
                 arguments."
            )));
        };
        self(I).into_result()
    }
}

#[allow(non_snake_case, unused_mut)]
impl<'a, F, Output, I1, I2> CliHandler<'a, ((I1, I2),)> for F
where
    F: FnOnce(I1, I2) -> Output,
    Output: IntoCliResult,
    I1: FromCliArgs<'a>,
    I2: FromCliArgs<'a>,
{
    fn call(self, args: &'a mut CollectedArgs) -> Result<(), CliError> {
        let handler_name = type_name::<Self>();

        let type_1_name = type_name::<I1>();
        let Some(I1) = I1::from_args(args) else {
            return Err(CliError::InvalidHandler(format!(
                "[{handler_name}]: Type `{type_1_name}` was not collected \
                 from the input arguments. Please double check the command \
                 structure!"
            )));
        };

        let type_2_name = type_name::<I2>();
        let Some(I2) = I2::from_args(args) else {
            return Err(CliError::InternalError(format!(
                "[{handler_name}]: Type `{type_2_name}` was not collected \
                 from the input arguments. Please double check the command \
                 structure!"
            )));
        };
        self(I1, I2).into_result()
    }
}

/// Compile-time test case.
const _: () = {
    #[derive(Clone)]
    struct CommonOpts;

    impl<'a> FromCliArgs<'a> for CommonOpts {
        fn from_args(args: &'a CollectedArgs) -> Option<Self> {
            args.get::<Self>().cloned()
        }
    }

    fn handle<'a, X, T: CliHandler<'a, X>>(
        args: &'a mut CollectedArgs,
        handler: T,
    ) {
        handler.call(args).unwrap();
    }

    fn test_empty_functions() {
        // returns Unit.
        fn noop() {}
        let mut args = CollectedArgs::default();
        handle(&mut args, noop);
    }

    fn test_result_functions() {
        fn noop() -> Result<(), anyhow::Error> {
            Ok(())
        }
        let mut args = CollectedArgs::default();
        handle(&mut args, noop);
    }

    fn test_functions_with_1_arg() {
        fn noop(_opts: CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        let mut args = CollectedArgs::default();
        handle(&mut args, noop);
    }

    // // Test to see if we can allow handlers to take reference instead.
    fn test_functions_with_1_ref() {
        fn noop(_opts: &CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        let mut args = CollectedArgs::default();
        handle(&mut args, noop);
    }
};
