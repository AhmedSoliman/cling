use darling::{FromDeriveInput, FromField, FromVariant};
//
// Attributes for struct/enum level #[cling(...)]
#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(cling), supports(struct_named, enum_newtype, enum_unit))]
pub(crate) struct CliRunnableAttrs {
    pub ident: syn::Ident,
    pub data: darling::ast::Data<EnumVariantAttrs, StructFieldAttrs>,
    pub generics: syn::Generics,
    /// Which handler function to run for this command
    pub run: Option<syn::Path>,
}

// Attributes for struct-field level #[cling(...)]
#[derive(Debug, Clone, FromField)]
#[darling(attributes(cling, command))]
pub(crate) struct StructFieldAttrs {
    // automatically populated by darling
    pub ident: Option<syn::Ident>,
    pub ty: syn::Type,

    #[darling(default)]
    #[allow(unused)]
    pub flatten: bool,

    #[darling(default)]
    pub subcommand: bool,
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes())]
pub(crate) struct VariantFieldAttrs {
    // automatically populated by darling
}
// Attributes for enum-variant level #[cling(...)]
#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(cling, command))]
pub(crate) struct EnumVariantAttrs {
    // automatically populated by darling
    pub ident: syn::Ident,
    pub fields: darling::ast::Fields<VariantFieldAttrs>,
    /// Which handler function to run for this command
    pub run: Option<syn::Path>,
}

// Attributes for derive CliParam
#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(), supports(any))]
pub(crate) struct CliParamAttrs {
    pub ident: syn::Ident,
}
