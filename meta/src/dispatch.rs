use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, Variant};

/// Check if a variant has the `#[dispatch(translate_discard)]` attribute
fn has_translate_discard(variant: &Variant) -> bool {
    variant.attrs.iter().any(|attr| {
        attr.path().is_ident("dispatch")
            && attr
                .parse_args::<Ident>()
                .map(|ident| ident == "translate_discard")
                .unwrap_or(false)
    })
}

/// Validate that a variant is a tuple variant with exactly one field
fn validate_variant(variant: &Variant) -> Result<(), Error> {
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(()),
        _ => Err(Error::new_spanned(
            variant,
            "StatementDispatch requires tuple variants with exactly one field",
        )),
    }
}

/// Generate the StatementDispatch implementations for an enum
pub fn generate_dispatch(input: &DeriveInput) -> TokenStream {
    generate_dispatch_inner(input).unwrap_or_else(|err| err.to_compile_error())
}

fn generate_dispatch_inner(input: &DeriveInput) -> Result<TokenStream, Error> {
    let enum_name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Err(Error::new_spanned(
                input,
                "StatementDispatch can only be derived for enums",
            ))
        }
    };

    for variant in variants {
        validate_variant(variant)?;
    }

    let typecheck_arms = variants.iter().map(|v| {
        let name = &v.ident;
        quote! { #enum_name::#name(inner) => inner.typecheck(meta) }
    });

    let translate_arms = variants.iter().map(|v| {
        let name = &v.ident;
        if has_translate_discard(v) {
            quote! {
                #enum_name::#name(inner) => {
                    inner.translate(meta);
                    FragmentKind::Empty
                }
            }
        } else {
            quote! { #enum_name::#name(inner) => inner.translate(meta) }
        }
    });

    let document_arms = variants.iter().map(|v| {
        let name = &v.ident;
        quote! { #enum_name::#name(inner) => inner.document(meta) }
    });

    Ok(quote! {
        impl TypeCheckModule for #enum_name {
            fn typecheck(&mut self, meta: &mut ParserMetadata) -> SyntaxResult {
                match self {
                    #(#typecheck_arms),*
                }
            }
        }

        impl TranslateModule for #enum_name {
            fn translate(&self, meta: &mut TranslateMetadata) -> FragmentKind {
                match self {
                    #(#translate_arms),*
                }
            }
        }

        impl DocumentationModule for #enum_name {
            fn document(&self, meta: &ParserMetadata) -> String {
                match self {
                    #(#document_arms),*
                }
            }
        }
    })
}
