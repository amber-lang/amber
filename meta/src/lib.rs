mod dispatch;
mod helper;
mod manager;
mod utils;

use crate::helper::HelperVisitor;
use crate::manager::ManagerVisitor;
use proc_macro::TokenStream;
use syn::visit::Visit;
use syn::{parse_macro_input, Attribute, DeriveInput};

/// Derive macro `ContextManager` allows changes to be made to annotated
/// fields on a struct, with automatic reset on early error return.
///
/// In this example, we change some settings on an object, and rely on
/// the context manager to reset those settings when it fails.  The macro
/// creates three functions for each annotated field in the `Amplifier`
/// struct, and we call the following ones here:
///
/// * Function `Amplifier::with_panel_ref()` swaps the existing `panel`
///   field on the `Amplifier` object, passes the `Amplifier` object to
///   the lambda by mutable reference, swaps the old `panel` field on
///   exit, and returns the result.
///
/// * Function `Amplifier::with_power()` sets the `power` field on the
///   `Amplifier` object, and resets the old value on exit.  Requires
///   the field being modified to implement the `Copy` and `Clone` traits.
///
/// * Function `Amplifier::with_panel_fn()` sets the `volume` field on
///   the encapsulated `Panel` object, by calling its setter function
///   `Panel::set_volume()`, and resets the old value on exit.  Note,
///   the setter function is created by derive macro `ContextHelper`.
///
/// ```rust
/// use amber_meta::{ContextHelper, ContextManager};
///
/// #[derive(ContextManager)]
/// struct Amplifier {
///     #[context]
///     power: bool,
///     input: f64,
///     output: f64,
///     #[context]
///     panel: Panel,
/// }
///
/// #[derive(ContextHelper)]
/// struct Panel {
///     #[context]
///     volume: u8,
///     display: Option<String>,
/// }
///
/// impl Panel {
///     fn new() -> Panel {
///         Panel { volume: 0, display: None }
///     }
/// }
///
/// fn demo_amplifier(amp: &mut Amplifier) -> Result<(), String> {
///     // Install a new control panel.
///     let mut panel = Panel::new();
///     amp.with_panel_ref(&mut panel, |amp| {
///         // Turn the power on.
///         amp.with_power(true, |amp| {
///             // Set the volume to 11.
///             amp.with_panel_fn(Panel::set_volume, 11, |amp| {
///                 // Strum a guitar chord.
///                 play_guitar(amp)?;
///                 Ok(())
///             })?;
///             // Reset the volume on exit.
///             Ok(())
///         })?;
///         // Turn the power off on exit.
///         Ok(())
///     })?;
///     // Reinstall the old control panel on exit.
///     Ok(())
/// }
///
/// fn play_guitar(amp: &Amplifier) -> Result<(), String> {
///     Err(String::from("Blown fuse"))
/// }
/// ```
#[proc_macro_derive(ContextManager, attributes(context))]
pub fn context_manager(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let syn::DeriveInput { data, ident, .. } = input;

    if !matches!(&data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(
            &ident,
            "ContextManager only supports structs, not enums or unions",
        )
        .into_compile_error()
        .into();
    }

    let mut visitor = ManagerVisitor::new(&ident);
    visitor.visit_data(&data);
    let output = visitor.make_block();
    TokenStream::from(output)
}

/// Derive macro `ContextHelper` provides support functions for use with
/// context functions created by `ContextManager`; for more information,
/// see documentation for that macro.
#[proc_macro_derive(ContextHelper, attributes(context))]
pub fn context_helper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let syn::DeriveInput { data, ident, .. } = input;

    if !matches!(&data, syn::Data::Struct(_)) {
        return syn::Error::new_spanned(
            &ident,
            "ContextHelper only supports structs, not enums or unions",
        )
        .into_compile_error()
        .into();
    }

    let mut visitor = HelperVisitor::new(&ident);
    visitor.visit_data(&data);
    let output = visitor.make_block();
    TokenStream::from(output)
}

/// Derive macro `StatementDispatch` generates trait implementations for
/// `TypeCheckModule`, `TranslateModule`, and `DocumentationModule` for
/// statement enums.
///
/// Each enum variant must be a tuple variant with exactly one field.
/// The generated implementations dispatch to the inner type's implementation.
///
/// Use `#[dispatch(translate_discard)]` on a variant to make its
/// `translate` method discard the result and return `FragmentKind::Empty`.
///
/// # Compile-time errors
///
/// The macro will fail to compile if applied to a struct:
///
/// ```compile_fail
/// use amber_meta::StatementDispatch;
///
/// #[derive(StatementDispatch)]
/// struct Test;
/// ```
///
/// The macro will also fail if applied to an enum with unit variants:
///
/// ```compile_fail
/// use amber_meta::StatementDispatch;
///
/// #[derive(StatementDispatch)]
/// enum InvalidVariant {
///     Unit,
/// }
/// ```
#[proc_macro_derive(StatementDispatch, attributes(dispatch))]
pub fn statement_dispatch(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let output = dispatch::generate_dispatch(&input);
    TokenStream::from(output)
}

/// Derive macro `AutoKeyword` requires the `#[keyword = "..."]` attribute.
/// Optionally supports `#[kind = "..."]` for specifying the keyword kind.
///
/// ```rust
/// use amber_meta::AutoKeyword;
///
/// #[derive(AutoKeyword)]
/// #[keyword = "if"]
/// struct IfCondition;
///
/// #[derive(AutoKeyword)]
/// #[keyword = "fun"]
/// #[kind = "stmt"]
/// struct FunKeyword;
/// ```
#[proc_macro_derive(AutoKeyword, attributes(keyword, kind))]
pub fn auto_keyword(input: TokenStream) -> TokenStream {
    use quote::quote;

    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    // Extract generics for proper impl generation
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let custom_keyword = match parse_keyword_attribute(&input.attrs) {
        Some(kw) => kw,
        None => {
            let err = syn::Error::new(
                name.span(),
                "AutoKeyword requires #[keyword = \"...\"] attribute",
            );
            return TokenStream::from(err.into_compile_error());
        }
    };

    let keyword_str = custom_keyword;

    let custom_kind = parse_kind_attribute(&input.attrs);

    let ty = quote! { #name #ty_generics };
    let keyword_lit = quote! { #keyword_str };

    let kind_expression = match custom_kind {
        Some(kind_str) => {
            let kind_str_lower = kind_str.to_lowercase();
            match kind_str_lower.as_str() {
                "stmt" | "statement" => {
                    quote! { crate::modules::keywords::KeywordKind::Stmt }
                }
                "builtin_stmt" | "builtin-stmt" | "builtinstatement" => {
                    quote! { crate::modules::keywords::KeywordKind::BuiltinStmt }
                }
                "builtin_expr" | "builtin-expr" | "builtinexpr" => {
                    quote! { crate::modules::keywords::KeywordKind::BuiltinExpr }
                }
                "binary_op" | "binary-op" | "binaryop" => {
                    quote! { crate::modules::keywords::KeywordKind::BinaryOp }
                }
                _ => {
                    let err = syn::Error::new(
                        name.span(),
                        format!(
                            "Invalid kind '{}'. Expected one of: stmt, builtin_stmt, builtin_expr, binary_op",
                            kind_str
                        )
                    );
                    return TokenStream::from(err.into_compile_error());
                }
            }
        }
        None => {
            quote! { crate::modules::keywords::KeywordKind::Stmt }
        }
    };

    let expanded = quote! {
        impl #impl_generics crate::modules::keywords::KeywordStmt for #ty #where_clause {
            fn keyword_stmt() -> &'static str {
                #keyword_lit
            }
        }

        impl #impl_generics crate::modules::keywords::KeywordExpr for #ty #where_clause {
            fn keyword_expr() -> &'static str {
                #keyword_lit
            }
        }

        impl #impl_generics crate::modules::keywords::BuiltinName for #ty #where_clause {
            fn builtin_name() -> &'static str {
                #keyword_lit
            }
        }

        // Register for auto-discovery via inventory
        inventory::submit! {
            crate::modules::keywords::KeywordRegistration::new_with_kind(
                stringify!(#name #ty_generics),
                #keyword_lit,
                #kind_expression,
            )
        }
    };

    TokenStream::from(expanded)
}

/// Parse the `#[kind = "..."]` attribute
fn parse_kind_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("kind") {
            let meta = &attr.meta;
            if let syn::Meta::NameValue(name_value) = meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &name_value.value
                {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

/// Parse the `#[keyword = "..."]` attribute
fn parse_keyword_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("keyword") {
            // Parse the attribute value
            let meta = &attr.meta;
            if let syn::Meta::NameValue(name_value) = meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &name_value.value
                {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}
