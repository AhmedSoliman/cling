use darling::util::parse_attribute_to_meta_list;
use darling::{FromDeriveInput, FromField, FromVariant};
//
// Attributes for struct/enum level #[cling(...)]
#[derive(Debug, Clone, FromDeriveInput)]
#[darling(
    attributes(cling),
    supports(struct_named, struct_unit, enum_newtype, enum_unit),
    forward_attrs(command, clap)
)]
pub(crate) struct CliRunnableAttrs {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<EnumVariantAttrs, StructFieldAttrs>,
    pub generics: syn::Generics,
    /// Which handler function to run for this command
    pub run: Option<syn::Path>,
}

// Attributes for struct-field level #[cling(...)]
#[derive(Debug, Clone, FromField)]
#[darling(attributes(cling), forward_attrs(command, clap))]
pub(crate) struct StructFieldAttrs {
    // automatically populated by darling
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    #[darling(default)]
    #[allow(unused)]
    pub flatten: bool,

    // #[darling(default)]
    // pub subcommand: bool,
    pub attrs: Vec<syn::Attribute>,
}

impl StructFieldAttrs {
    pub fn is_subcommand(&self) -> bool {
        for attr in &self.attrs {
            if attr.path().is_ident("command") || attr.path().is_ident("clap") {
                if let Ok(meta_list) = parse_attribute_to_meta_list(attr) {
                    if let Ok(arg) = meta_list.parse_args::<syn::Ident>() {
                        if arg == "subcommand" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(), forward_attrs(command, clap))]
pub(crate) struct VariantFieldAttrs {
    // automatically populated by darling
}
// Attributes for enum-variant level #[cling(...)]
#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(cling), forward_attrs(command, clap))]
pub(crate) struct EnumVariantAttrs {
    // automatically populated by darling
    pub ident: syn::Ident,
    pub fields: darling::ast::Fields<VariantFieldAttrs>,
    /// Which handler function to run for this command
    pub run: Option<syn::Path>,
}

// Attributes for derive CliParam
#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(), supports(any), forward_attrs(command, clap))]
pub(crate) struct CliParamAttrs {
    pub ident: syn::Ident,
}
