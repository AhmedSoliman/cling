use crate::args::{CollectedArgs, FromCliArgs};
use crate::prelude::CliError;

pub trait CliHandler<Input> {
    fn call(self, args: &mut CollectedArgs) -> Result<(), CliError>;
}

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

impl<F, Output> CliHandler<((),)> for F
where
    F: FnOnce() -> Output,
    Output: IntoCliResult,
{
    fn call(self, _args: &mut CollectedArgs) -> Result<(), CliError> {
        self().into_result()
    }
}

#[allow(non_snake_case, unused_mut)]
impl<F, Output, I> CliHandler<((I,),)> for F
where
    F: FnOnce(I) -> Output,
    Output: IntoCliResult,
    I: FromCliArgs,
{
    fn call(self, args: &mut CollectedArgs) -> Result<(), CliError> {
        let Some(I) = I::from_args(args) else {
            return Err(CliError::InternalError(
                "Type $ty couldn't be collected from the input arguments."
                    .to_owned(),
            ));
        };
        self(I).into_result()
    }
}

#[allow(non_snake_case, unused_mut)]
impl<F, Output, I1, I2> CliHandler<((I1, I2),)> for F
where
    F: FnOnce(I1, I2) -> Output,
    Output: IntoCliResult,
    I1: FromCliArgs,
    I2: FromCliArgs,
{
    fn call(self, args: &mut CollectedArgs) -> Result<(), CliError> {
        let Some(I1) = I1::from_args(args) else {
            return Err(CliError::InternalError(
                "Type I1 couldn't be collected from the input arguments."
                    .to_owned(),
            ));
        };

        let Some(I2) = I2::from_args(args) else {
            return Err(CliError::InternalError(
                "Type I2 couldn't be collected from the input arguments."
                    .to_owned(),
            ));
        };
        self(I1, I2).into_result()
    }
}

/// Compile-time test case.
const _: () = {
    struct CommonOpts;

    impl FromCliArgs for CommonOpts {
        fn from_args(args: &mut CollectedArgs) -> Option<Self> {
            args.get()
        }
    }

    // Test to see if we can allow handlers to take reference instead.
    impl FromCliArgs for &CommonOpts {
        fn from_args(args: &mut CollectedArgs) -> Option<Self> {
            args.get()
        }
    }

    // Test to see if we can allow handlers to take mutable reference instead.
    impl FromCliArgs for &mut CommonOpts {
        fn from_args(args: &mut CollectedArgs) -> Option<Self> {
            args.get()
        }
    }

    fn handle<X, T: CliHandler<X>>(handler: T) {
        let mut args = CollectedArgs {};
        let _ = handler.call(&mut args);
    }

    fn test_empty_functions() {
        // returns Unit.
        fn noop() {}
        handle(noop);
    }

    fn test_result_functions() {
        fn noop() -> Result<(), anyhow::Error> {
            Ok(())
        }
        handle(noop);
    }

    fn test_functions_with_1_arg() {
        fn noop(_opts: CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        handle(noop);
    }

    fn test_functions_with_1_ref() {
        fn noop(_opts: &CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        handle(noop);
    }

    fn test_functions_with_1_mut_ref() {
        fn noop(_opts: &mut CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        handle(noop);
    }
};
