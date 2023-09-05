use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, ReturnType};

pub fn expand_handler(input: &ItemFn) -> TokenStream {
    let input_types = &input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                | FnArg::Typed(pat_type) => Some(&*pat_type.ty),
                | FnArg::Receiver(_) => None,
            }
        })
        .collect::<Vec<_>>();
    let output_check = if let ReturnType::Type(_, ret_type) = &input.sig.output
    {
        let span = ret_type.span();
        quote::quote_spanned! { span =>
            assert_cling_into_effect::<#ret_type, _>();
        }
    } else {
        quote::quote! {}
    };

    let input_checks = input_types
        .iter()
        .map(|ty| {
            let span = ty.span();
            quote::quote_spanned! { span =>
                assert_cling_param_type::<#ty>();
            }
        })
        .collect::<Vec<_>>();

    quote::quote! {
         const _: () = {
            const fn assert_cling_into_effect<T: ::cling::_private::IntoEffect<A>, A>() {}
            const fn assert_cling_param_type<'a, T: ::cling::_private::HandlerParam<'a>>() {}

            // Validate that output type implements IntoEffect
            #output_check
            // Validate that function inputs implement HandlerParam
            #(#input_checks)*
         };

        #input
    }
}
