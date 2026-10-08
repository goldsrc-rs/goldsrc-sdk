//! Event subscriber handler parsing and code generation.

use crate::utils::check_sig_args;
use proc_macro2::TokenStream;
use quote::quote;

/// Converts a CamelCase identifier to snake_case wire name.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            for lc in ch.to_lowercase() {
                result.push(lc);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Extracted event handler information.
#[derive(Debug)]
pub struct EventHandler {
    /// Canonical event wire name string (e.g. `"round_start"`).
    pub name: String,
    /// Target event variant identifier (e.g. `RoundStart`).
    pub variant_ident: syn::Ident,
    /// Handler function identifier.
    pub ident: syn::Ident,
    /// Number of parameters.
    pub inputs_len: usize,
}

/// Parses a `#[event]` attribute on a method.
///
/// Enforces compiler-checked `EngineEvent` enum variants (e.g. `#[event(EngineEvent::RoundStart)]`
/// or `#[event(RoundStart)]`). String literals are rejected at compile time as AMX Mod X legacy.
pub fn parse_event(attr: &syn::Attribute, sig: &syn::Signature) -> syn::Result<EventHandler> {
    check_sig_args(sig, "event", &[0, 1, 2])?;

    // Check if user passed a string literal directly e.g. #[event("round_start")]
    if let Ok(syn::Lit::Str(lit_str)) = attr.parse_args::<syn::Lit>() {
        return Err(syn::Error::new_spanned(
            lit_str,
            "string literals in #[event(...)] are legacy from AMX Mod X and have been removed; use typed `EngineEvent::Variant` or `Variant` (e.g. `#[event(EngineEvent::RoundStart)]` or `#[event(RoundStart)]`) instead",
        ));
    }

    let mut variant_ident = None;

    if let Ok(path) = attr.parse_args::<syn::Path>() {
        if let Some(seg) = path.segments.last() {
            variant_ident = Some(seg.ident.clone());
        }
    } else if let Ok(meta_list) = attr.meta.require_list() {
        let mut string_error = None;
        let _ = meta_list.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") || meta.path.is_ident("event") {
                let value = meta.value()?;
                let expr: syn::Expr = value.parse()?;
                match expr {
                    syn::Expr::Lit(lit) => {
                        string_error = Some(syn::Error::new_spanned(
                            lit,
                            "string literals in #[event(...)] are legacy from AMX Mod X and have been removed; use typed `EngineEvent::Variant` or `Variant` (e.g. `#[event(EngineEvent::RoundStart)]` or `#[event(RoundStart)]`) instead",
                        ));
                    }
                    syn::Expr::Path(expr_path) => {
                        if let Some(seg) = expr_path.path.segments.last() {
                            variant_ident = Some(seg.ident.clone());
                        }
                    }
                    other => {
                        string_error = Some(syn::Error::new_spanned(
                            other,
                            "expected typed event variant (e.g. `EngineEvent::RoundStart` or `RoundStart`)",
                        ));
                    }
                }
            }
            Ok(())
        });
        if let Some(err) = string_error {
            return Err(err);
        }
    }

    let variant = variant_ident.ok_or_else(|| {
        syn::Error::new_spanned(
            attr,
            "missing or invalid event target in #[event(...)]; specify typed event, e.g. `#[event(EngineEvent::RoundStart)]` or `#[event(RoundStart)]`",
        )
    })?;

    let wire_name = to_snake_case(&variant.to_string());

    Ok(EventHandler {
        name: wire_name,
        variant_ident: variant,
        ident: sig.ident.clone(),
        inputs_len: sig.inputs.len(),
    })
}

/// Generates event subscriber registrations for `on_load`.
pub fn generate_event_registrations(
    struct_name: &syn::Type,
    handlers: &[EventHandler],
) -> Vec<TokenStream> {
    let mut registrations = Vec::new();
    for h in handlers {
        let fn_name = &h.ident;
        let variant_ident = &h.variant_ident;
        let ev_name = &h.name;
        let call = match h.inputs_len {
            0 => quote! { |_payload| { #struct_name::#fn_name(); } },
            1 => quote! { |payload| { #struct_name::#fn_name(payload.to_vec()); } },
            _ => {
                quote! { |payload| { #struct_name::#fn_name(#ev_name.to_string(), payload.to_vec()); } }
            }
        };
        registrations.push(quote! {
            const _: () = {
                let _ = ::goldsrc::event::EngineEvent::#variant_ident;
            };
            ::goldsrc::event::Event::subscriber(::goldsrc::event::EngineEvent::#variant_ident.as_str())
                .subscribe(#call);
        });
    }
    registrations
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, parse_quote};

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("RoundStart"), "round_start");
        assert_eq!(to_snake_case("PlayerPostThink"), "player_post_think");
        assert_eq!(to_snake_case("ServerFrame"), "server_frame");
        assert_eq!(to_snake_case("CmdStart"), "cmd_start");
        assert_eq!(
            to_snake_case("ClientUserInfoChanged"),
            "client_user_info_changed"
        );
    }

    #[test]
    fn test_parse_event_path() {
        let attr: Attribute = parse_quote!(#[event(EngineEvent::RoundStart)]);
        let sig: syn::Signature = parse_quote!(fn on_round_start());
        let handler = parse_event(&attr, &sig).unwrap();
        assert_eq!(handler.name, "round_start");
        assert_eq!(handler.variant_ident, "RoundStart");
        assert_eq!(handler.inputs_len, 0);
    }

    #[test]
    fn test_parse_event_bare_ident() {
        let attr: Attribute = parse_quote!(#[event(PlayerPostThink)]);
        let sig: syn::Signature = parse_quote!(fn on_post_think(payload: Vec<u8>));
        let handler = parse_event(&attr, &sig).unwrap();
        assert_eq!(handler.name, "player_post_think");
        assert_eq!(handler.variant_ident, "PlayerPostThink");
        assert_eq!(handler.inputs_len, 1);
    }

    #[test]
    fn test_parse_event_nested_meta() {
        let attr: Attribute = parse_quote!(#[event(name = RoundStart)]);
        let sig: syn::Signature = parse_quote!(fn on_round_start());
        let handler = parse_event(&attr, &sig).unwrap();
        assert_eq!(handler.name, "round_start");
        assert_eq!(handler.variant_ident, "RoundStart");
    }

    #[test]
    fn test_rejects_string_literal_legacy() {
        let attr: Attribute = parse_quote!(#[event("round_start")]);
        let sig: syn::Signature = parse_quote!(fn on_round_start());
        let err = parse_event(&attr, &sig).unwrap_err();
        assert!(
            err.to_string()
                .contains("string literals in #[event(...)] are legacy from AMX Mod X")
        );
    }

    #[test]
    fn test_rejects_nested_string_literal_legacy() {
        let attr: Attribute = parse_quote!(#[event(name = "round_start")]);
        let sig: syn::Signature = parse_quote!(fn on_round_start());
        let err = parse_event(&attr, &sig).unwrap_err();
        assert!(
            err.to_string()
                .contains("string literals in #[event(...)] are legacy from AMX Mod X")
        );
    }
}
