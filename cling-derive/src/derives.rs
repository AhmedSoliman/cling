use darling::ast::Fields;
use darling::{Error, FromDeriveInput};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{DeriveInput, Ident};

use crate::attributes::{
    CliParamAttrs,
    CliRunnableAttrs,
    EnumVariantAttrs,
    StructFieldAttrs,
};

pub fn derive_cli_runnable(input: &DeriveInput) -> TokenStream {
    let attrs = match CliRunnableAttrs::from_derive_input(input) {
        | Ok(attrs) => attrs,
        | Err(e) => {
            return e.write_errors();
        }
    };

    expand(attrs)
}

pub fn derive_cli_param(input: &DeriveInput) -> TokenStream {
    let attrs = match CliParamAttrs::from_derive_input(input) {
        | Ok(attrs) => attrs,
        | Err(e) => {
            return e.write_errors();
        }
    };

    let name = &attrs.ident;
    //let generics = &attrs.generics;
    gen_from_cli_param_impl(name)
}

fn gen_from_cli_param_impl(name: &Ident) -> TokenStream {
    quote::quote! {
        impl<'a> cling::args::CliParam<'a> for #name {
            fn from_args(args: &'a cling::args::CollectedArgs) -> core::option::Option<Self> {
                args.get::<Self>().cloned()
            }
        }
    }
}

fn expand(attrs: CliRunnableAttrs) -> TokenStream {
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
    attrs: &CliRunnableAttrs,
    fields: &Fields<StructFieldAttrs>,
) -> darling::Result<TokenStream> {
    let mut acc = darling::Error::accumulator();

    let span = attrs.run.span();
    let run_self = match &attrs.run {
        // We have a handler for this runnable, let's make sure we execute it.
        | Some(run) => {
            quote::quote_spanned! { span =>
                cling::handler::CliHandler::call(#run, args)?.into_result().await?;
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
            args.insert(self.clone());
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
        if field.is_subcommand() {
            let span = field.ty.span();
            found_subcommand = true;
            // We assume that it's a CliRunnable as well.
            subcommand_runs.extend(quote::quote_spanned! { span =>
                <dyn ::cling::prelude::CliRunnable>::call(&self.#field_name, args).await?;
            });
        } else {
            // Not a subcommand, let's see if we should collect it.
            collect_arguments.extend(quote::quote! {
                if (&self.#field_name).as_collectable().can_collect() {
                    args.insert(self.#field_name.clone());
                }
            });
        }
    }

    if run_self.is_empty() && !found_subcommand {
        // We don't have any subcommands, and we don't have a #[cling(run =
        // ...)]. In practice, the user might have defined their sub-commands in
        // a flattened member, but it's tricky to figure that out. We
        // will fail, but we might come back to this in the future and
        // provide better heuristic.
        acc.push(Error::custom(
            "CliRunnable must have a #[cling(run = ...)] attribute or a \
             subcommand field",
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
    attrs: &CliRunnableAttrs,
    variants: &Vec<EnumVariantAttrs>,
) -> darling::Result<TokenStream> {
    if attrs.run.is_some() {
        return Err(Error::custom(
            "Runnable enum cannot have a #[cling(run = ...)] attribute. \
             Please mark the unit variants with #[cling(run = ...)] instead \
             and/or derive CliRunnable for the variant newtype argument",
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
                            cling::handler::CliHandler::call(#run, args)?.into_result().await?;
                        }
                    });
                }
                | None => {
                    acc.push(Error::custom(
                        "Unit enum variants must have a #[cling(run = ...)] \
                         attribute",
                    ));
                }
            }
        } else if variant.run.is_some() {
            acc.push(
                Error::custom(
                    "Non-unit enums cannot have #[cling(run = ...)]. Instead, \
                     derive CliRunnable on the variant newtype argument as \
                     usual.",
                )
                .with_span(&variant.run),
            );
        } else {
            // We will dispatch to the newtype assuming that it's CliRunnable
            variant_tokens.push(quote::quote_spanned! { span =>
                #enum_name::#variant_name(sub) => {
                    <dyn ::cling::prelude::CliRunnable>::call(sub, args).await?;
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

fn gen_runnable_impl(
    attrs: &CliRunnableAttrs,
    impl_body: TokenStream,
) -> TokenStream {
    let name = &attrs.ident;
    let generics = &attrs.generics;
    quote::quote! {
        #[automatically_derived]
        #[allow(clippy::all)]
        #[::cling::async_trait]
        impl #generics ::cling::prelude::CliRunnable for #name #generics {
            async fn call(
                &self,
                args: &mut cling::args::CollectedArgs,
            ) -> std::result::Result<(), cling::prelude::CliError> {
                use cling::args::{CollectableKind, CollectedArgs, UnknownKind};
                use cling::handler::*;

                #impl_body
                Ok(())
            }
        }
    }
}
