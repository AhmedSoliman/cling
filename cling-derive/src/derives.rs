use darling::ast::Fields;
use darling::{Error, FromDeriveInput};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::DeriveInput;

use crate::attributes::{
    CollectAttrs,
    EnumVariantAttrs,
    RunAttrs,
    StructFieldAttrs,
};

pub fn derive_run(input: &DeriveInput) -> TokenStream {
    let attrs = match RunAttrs::from_derive_input(input) {
        | Ok(attrs) => attrs,
        | Err(e) => {
            return e.write_errors();
        }
    };

    expand(attrs)
}

pub fn derive_collect(input: &DeriveInput) -> TokenStream {
    let attrs = match CollectAttrs::from_derive_input(input) {
        | Ok(attrs) => attrs,
        | Err(e) => {
            return e.write_errors();
        }
    };

    let name = &attrs.ident;
    quote::quote! {
        impl<'a> cling::_private::Collect for #name {}
    }
}

fn expand(attrs: RunAttrs) -> TokenStream {
    let tokens = match &attrs.data {
        | darling::ast::Data::Enum(variants) => expand_enum(&attrs, variants),
        | darling::ast::Data::Struct(fields) => expand_struct(&attrs, fields),
    };

    match tokens {
        | Ok(tokens) => tokens,
        | Err(e) => e.write_errors(),
    }
}

fn expand_struct(
    attrs: &RunAttrs,
    fields: &Fields<StructFieldAttrs>,
) -> darling::Result<TokenStream> {
    let mut acc = darling::Error::accumulator();

    let type_ident = &attrs.ident;
    let span = attrs.run.span();
    let run_self = match &attrs.run {
        // We have a handler for this runnable, let's make sure we execute it.
        | Some(run) => {
            quote::quote_spanned! { span =>
                ::cling::_private::tracing::log::debug!(
                    target: "cling",
                    "Running handler `{}` of type `{}`",
                    stringify!(#run),
                    stringify!(#type_ident),
                );
                {
                    let effect = cling::_private::Handler::call(#run, args)?.into_effect().await?;
                    effect.apply_effect(args);
                }
            }
        }
        | None => quote::quote!(),
    };

    // collect _Collectable_ fields in CollectedArgs
    let mut collect_arguments = TokenStream::new();
    let mut subcommand_runs = TokenStream::new();
    let mut found_subcommand = false;
    // We collect our own object in all cases.
    collect_arguments.extend(quote::quote! {
         if (self).as_collectable().can_collect() {
             args.insert(self.clone(), false);
         }
    });
    for field in &fields.fields {
        if found_subcommand {
            acc.push(Error::custom(
                "Subcommand fields can only be used once in a struct",
            ));
        }

        // We only support named structs. darling validation will ensure this.
        let field_name = field.ident.clone().unwrap();
        let field_type = &field.ty;
        if field.is_subcommand() {
            let span = field.ty.span();
            found_subcommand = true;
            // We assume that it's Run as well.
            subcommand_runs.extend(quote::quote_spanned! { span =>
                <dyn ::cling::prelude::Run>::call(&self.#field_name, args).await?;
            });
        } else {
            // Escape hatch if this particular field is problematic.
            if field.skip {
                collect_arguments.extend(quote::quote! {
                    ::cling::_private::tracing::log::debug!(
                        target: "cling",
                        "Skipping `{}.{}` because it's marked with `#[cling(skip)]`",
                        stringify!(#type_ident),
                        stringify!(#field_name),
                    );
                });
            } else if field.collect {
                // If the field is marked with #[cling(collect)], we will wrap
                // it in Collected<T> and store it wrapped.
                collect_arguments.extend(quote::quote! {
                    ::cling::_private::tracing::log::debug!(
                        target: "cling",
                        "Collecting type `{}` from `{}.{}` \
                            because it's marked with `#[cling(collect)]`. \
                            This can be extracted with cling::prelude::Collected<T> at runtime.",
                        stringify!(#field_type),
                        stringify!(#type_ident),
                        stringify!(#field_name),
                    );
                    args.insert(::cling::prelude::Collected(self.#field_name.clone()), false);
                });
            } else {
                // Not a subcommand, let's see if we should collect it.
                collect_arguments.extend(quote::quote! {
                    if (&self.#field_name).as_collectable().can_collect() {
                        ::cling::_private::tracing::log::debug!(
                            target: "cling",
                            "Collecting type `{}` from `{}.{}` because it derives `Collect`",
                            stringify!(#field_type),
                            stringify!(#type_ident),
                            stringify!(#field_name),
                        );
                        args.insert(self.#field_name.clone(), false);
                    } else {
                        ::cling::_private::tracing::log::trace!(
                            target: "cling",
                            "Skipping `{}.{}` because `{}` doesn't implement `Collect`",
                            stringify!(#type_ident),
                            stringify!(#field_name),
                            stringify!(#field_type),
                        );
                    }
                });
            }
        }
    }

    if run_self.is_empty() && !found_subcommand {
        // We don't have any subcommands, and we don't have a #[cling(run =
        // ...)]. In practice, the user might have defined their sub-commands in
        // a flattened member, but it's tricky to figure that out. We
        // will fail, but we might come back to this in the future and
        // provide better heuristic.
        acc.push(Error::custom(
            "must have a #[cling(run = ...)] attribute or a clap(subcommand) \
             field",
        ));
        return acc.finish_with(TokenStream::new());
    }
    // runnable_impl
    let impl_runnable = gen_runnable_impl(
        attrs,
        quote::quote! {
            // Collect fields that are collectable
            #collect_arguments
            // run self if run attribute is defined
            #run_self
            // run subcommands if any
            #subcommand_runs
        },
    );

    acc.finish_with(impl_runnable)
}

/// Expanding for sub-commands enum
fn expand_enum(
    attrs: &RunAttrs,
    variants: &Vec<EnumVariantAttrs>,
) -> darling::Result<TokenStream> {
    if attrs.run.is_some() {
        return Err(Error::custom(
            "Runnable enum cannot have a #[cling(run = ...)] attribute. \
             Please mark the unit variants with #[cling(run = ...)] instead \
             and/or derive Run for the variant newtype argument",
        ));
    }

    let mut acc = darling::Error::accumulator();

    let mut variant_tokens = Vec::with_capacity(variants.len());

    let enum_name = &attrs.ident;
    for variant in variants {
        let span = variant.ident.span();
        let variant_name = &variant.ident;
        if variant.fields.is_empty() {
            // We must have a #[cling(run = ...)] attribute.
            match &variant.run {
                | Some(run) => {
                    variant_tokens.push(quote::quote_spanned! { span =>
                        #enum_name::#variant_name => {
                            ::cling::_private::tracing::log::debug!(
                                target: "cling",
                                "Running handler `{}` of variant `{}::{}`",
                                stringify!(#run),
                                stringify!(#enum_name),
                                stringify!(#variant_name),
                            );
                            {
                                let effect = cling::_private::Handler::call(#run, args)?.into_effect().await?;
                                effect.apply_effect(args);
                            }
                        }
                    });
                }
                | None => {
                    acc.push(
                        Error::custom(
                            "Unit enum variants must have a #[cling(run = \
                             ...)] attribute",
                        )
                        .with_span(&variant.ident),
                    )
                }
            }
        } else if variant.run.is_some() {
            let var_inner_ty =
                &variant.fields.fields.first().expect("non-empty variant").ty;
            let var_inner_ty = quote::quote!(#var_inner_ty).to_string();
            acc.push(
                Error::custom(format!(
                    "Non-unit enums cannot have #[cling(run = ...)]. Instead, \
                     derive Run on the variant inner type `{}`.",
                    var_inner_ty,
                ))
                .with_span(&variant.run),
            );
        } else {
            // We will dispatch to the newtype assuming that it's Run
            variant_tokens.push(quote::quote_spanned! { span =>
                #enum_name::#variant_name(sub) => {
                    <dyn ::cling::prelude::Run>::call(sub, args).await?;
                }
            });
        }
    }
    let tokens = gen_runnable_impl(
        attrs,
        quote::quote! {
            match self {
                #(#variant_tokens)*
            }
        },
    );

    acc.finish_with(tokens)
}

fn gen_runnable_impl(attrs: &RunAttrs, impl_body: TokenStream) -> TokenStream {
    let name = &attrs.ident;
    let generics = &attrs.generics;
    quote::quote! {
        #[automatically_derived]
        #[allow(clippy::all)]
        impl #generics ::cling::prelude::Run for #name #generics {
            fn call<'a>(
                &'a self,
                args: &'a mut cling::_private::CollectedArgs,
            ) ->  std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), cling::prelude::CliError>> + Send + 'a>> {
                use cling::_private::*;

                Box::pin(async move {
                    #impl_body
                    Ok(())
                })
            }
        }
        ::cling::_private::static_assertions::assert_impl_all!(#name #generics: Clone);
    }
}
