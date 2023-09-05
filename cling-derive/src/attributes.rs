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
pub(crate) struct RunAttrs {
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
    pub collect: bool,

    // Escape hatch if our logic for collectable went wrong (fails to compile)
    // and the user wants to force-ignore processing a certain field.
    #[darling(default)]
    pub skip: bool,

    pub attrs: Vec<syn::Attribute>,
}

impl StructFieldAttrs {
    pub fn is_subcommand(&self) -> bool {
        has_subcommand(&self.attrs)
    }
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(), forward_attrs(command, clap))]
pub(crate) struct VariantFieldAttrs {
    pub ty: syn::Type,
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

// Attributes for derive Collect
#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(), supports(any), forward_attrs(command, clap))]
pub(crate) struct CollectAttrs {
    pub ident: syn::Ident,
}

fn has_subcommand(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
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
