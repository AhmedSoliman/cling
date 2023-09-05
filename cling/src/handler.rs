use std::any::type_name;

use indoc::formatdoc;

use crate::effects::HandlerEffect;
use crate::params::{CollectedArgs, HandlerParam};
use crate::prelude::CliError;
use crate::IntoEffect;

/// Trait for functions that handle command line commands.
pub trait Handler<'a, Type, Input, Output, F>
where
    F: HandlerEffect,
    Output: IntoEffect<Type, Effect = F>,
{
    fn call(self, args: &'a mut CollectedArgs) -> Result<Output, CliError>;
}

impl<'a, Type, F, Output, Effect> Handler<'a, Type, ((),), Output, Effect> for F
where
    F: FnOnce() -> Output + Send,
    Effect: HandlerEffect,
    Output: IntoEffect<Type, Effect = Effect>,
{
    fn call(self, _args: &'a mut CollectedArgs) -> Result<Output, CliError> {
        Ok(self())
    }
}

macro_rules! handler_impl {
    ($($ty:ident),* $(,)?) => {

        #[allow(non_snake_case, unused_mut)]
        impl<'a, Type, F, Output, $($ty),*, Effect> Handler<'a, Type, (($($ty,)*),), Output, Effect> for F
        where
            F: FnOnce($($ty,)*) -> Output + Send,
            Output: IntoEffect<Type, Effect = Effect>,
            Effect: HandlerEffect,
            $($ty: HandlerParam<'a> + Send),*
        {
            fn call(self, args: &'a mut CollectedArgs) -> Result<Output, CliError> {
                let handler_name = type_name::<Self>();

                $(
                let Some($ty) = $ty::extract_param(args) else {
                    let mut collected = args.collected_types();
                    collected.sort();
                    return Err(CliError::InvalidHandler(formatdoc!{"
                        In `{handler_name}`: Type `{}` was not collected from input arguments. Possible reasons:
                           - The type doesn't implement `Collect` (add #[derive(Collect)])
                           - The struct field wasn't marked with `#[cling(collect)]`
                           - The type is not present in any fields, enums, or structs leading to this command in the command hierarchy.
                           - The type is defined with Option<T> or Vec<T> and you used T, or vice versa

                           Those are the types that have been collected: {:#?}
                           "
                         ,
                             type_name::<$ty>(),
                             collected,
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
    impl crate::params::Collect for CommonOpts {}

    async fn handle<
        'a,
        Type,
        Output: IntoEffect<Type, Effect = F>,
        X,
        F: HandlerEffect,
        T: Handler<'a, Type, X, Output, F>,
    >(
        args: &'a mut CollectedArgs,
        handler: T,
    ) {
        crate::_private::Handler::call(handler, args)
            .unwrap()
            .into_effect()
            .await
            .unwrap();
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
        async fn noop(_opts: &CommonOpts) -> Result<(), anyhow::Error> {
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
