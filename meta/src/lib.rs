mod dispatch;
mod helper;
mod manager;
mod utils;

use crate::helper::HelperVisitor;
use crate::manager::ManagerVisitor;
use proc_macro::TokenStream;
use syn::visit::Visit;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

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
    let input = parse_macro_input!(input as ItemStruct);
    let mut visitor = ManagerVisitor::new(&input.ident);
    visitor.visit_item_struct(&input);
    let output = visitor.make_block();
    TokenStream::from(output)
}

/// Derive macro `ContextHelper` provides support functions for use with
/// context functions created by `ContextManager`; for more information,
/// see documentation for that macro.
#[proc_macro_derive(ContextHelper, attributes(context))]
pub fn context_helper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let mut visitor = HelperVisitor::new(&input.ident);
    visitor.visit_item_struct(&input);
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
