use std::any::type_name;
use std::future::Future;

use indoc::formatdoc;

use crate::args::{CliParam, CollectedArgs};
use crate::prelude::CliError;

pub struct _Sync;
pub struct _Async;

/// trait for functions that can be used to handle command line commands.
pub trait CliHandler<'a, Type, Input, Output>
where
    Output: IntoCliResult<Type>,
{
    fn call(self, args: &'a mut CollectedArgs) -> Result<Output, CliError>;
}

/// trait to handle function return types:
/// - Result<(), E> where E: Into<CliError>
/// - ()
#[async_trait::async_trait]
pub trait IntoCliResult<Type> {
    async fn into_result(self) -> Result<(), CliError>;
}

#[async_trait::async_trait]
impl IntoCliResult<_Sync> for () {
    async fn into_result(self) -> Result<(), CliError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl<E> IntoCliResult<_Sync> for Result<(), E>
where
    E: Into<CliError>,
    Self: Send,
{
    async fn into_result(self) -> Result<(), CliError> {
        self.map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl<T, Output> IntoCliResult<_Async> for T
where
    T: Future<Output = Output> + Send,
    Output: IntoCliResult<_Sync> + Send,
{
    async fn into_result(self) -> Result<(), CliError> {
        self.await.into_result().await
    }
}

// we want to handle functions that return:
// 1. Result<(), E> where E: Into<CliError>
// 2. ()
//
// So type that implements trait IntoCliResult is accepted.
impl<'a, Type, F, Output> CliHandler<'a, Type, ((),), Output> for F
where
    F: FnOnce() -> Output + Send,
    Output: IntoCliResult<Type>,
{
    fn call(self, _args: &'a mut CollectedArgs) -> Result<Output, CliError> {
        Ok(self())
    }
}

macro_rules! handler_impl {
    ($($ty:ident),* $(,)?) => {

        #[allow(non_snake_case, unused_mut)]
        impl<'a, Type, F, Output, $($ty),*> CliHandler<'a, Type, (($($ty,)*),), Output> for F
        where
            F: FnOnce($($ty,)*) -> Output + Send,
            Output: IntoCliResult<Type>,
            $($ty: CliParam<'a> + Send),*
        {
            fn call(self, args: &'a mut CollectedArgs) -> Result<Output, CliError> {
                let handler_name = type_name::<Self>();

                $(
                let Some($ty) = $ty::from_args(args) else {
                    return Err(CliError::InvalidHandler(formatdoc!{"
                        In `{handler_name}`: Type `{}` was not collected from input arguments. Possible reasons:
                           - The type doesn't implement `CliParam` (add derive(CliParam))
                           - The type is not a field in any type leading to this command
                           - The type is defined with Option<T> or Vec<T> and you used T, or vice versa
                           
                           Those are the types that have been collected: {:#?}
                           "
                         , type_name::<$ty>(),
                         args.collected_types()

                         }));
                };
                )*
                Ok(self($($ty),*))
            }
        }
    };
}

handler_impl!(T1);
handler_impl!(T1, T2);
handler_impl!(T1, T2, T3);
handler_impl!(T1, T2, T3, T4);
handler_impl!(T1, T2, T3, T4, T5);
handler_impl!(T1, T2, T3, T4, T5, T6);
handler_impl!(T1, T2, T3, T4, T5, T6, T7);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
handler_impl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
handler_impl!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);

/// Compile-time test case.
const _: () = {
    #[derive(Clone)]
    struct CommonOpts;

    impl<'a> CliParam<'a> for CommonOpts {
        fn from_args(args: &'a CollectedArgs) -> Option<Self> {
            args.get::<Self>().cloned()
        }
    }

    async fn handle<
        'a,
        Type,
        Output: IntoCliResult<Type>,
        X,
        T: CliHandler<'a, Type, X, Output>,
    >(
        args: &'a mut CollectedArgs,
        handler: T,
    ) {
        handler.call(args).unwrap().into_result().await.unwrap();
    }

    async fn test_empty_sync_functions() {
        // returns Unit.
        fn noop() {}
        let mut args = CollectedArgs::default();
        handle(&mut args, noop).await;
    }

    async fn test_empty_functions() {
        // returns Unit.
        async fn noop() {}
        let mut args = CollectedArgs::default();
        handle(&mut args, noop).await;
    }

    async fn test_result_functions() {
        async fn noop() -> Result<(), anyhow::Error> {
            Ok(())
        }
        let mut args = CollectedArgs::default();
        handle(&mut args, noop).await;
    }

    async fn test_functions_with_1_arg() {
        async fn noop(_opts: CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        let mut args = CollectedArgs::default();
        handle(&mut args, noop).await;
    }

    // // Test to see if we can allow handlers to take reference instead.
    async fn test_functions_with_1_ref() {
        async fn noop(_opts: &CommonOpts) -> Result<(), anyhow::Error> {
            Ok(())
        }
        let mut args = CollectedArgs::default();
        handle(&mut args, noop).await;
    }
};
