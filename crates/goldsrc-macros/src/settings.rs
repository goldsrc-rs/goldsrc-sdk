//! Procedural macro implementation for `#[derive(Settings)]`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

/// Expands the `#[derive(Settings)]` macro.
pub fn expand_derive_settings(input: DeriveInput) -> Result<TokenStream, Error> {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data_struct) = &input.data else {
        return Err(Error::new_spanned(
            &input,
            "Settings can only be derived for structs with named fields",
        ));
    };

    let Fields::Named(fields_named) = &data_struct.fields else {
        return Err(Error::new_spanned(
            &input,
            "Settings can only be derived for structs with named fields",
        ));
    };

    // Parse container-level #[settings(prefix = "...", section = "...")]
    let mut key_prefix = String::new();
    let mut default_section: Option<String> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("settings") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("prefix") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    key_prefix = value.value();
                } else if meta.path.is_ident("section") {
                    let value: syn::LitStr = meta.value()?.parse()?;
                    default_section = Some(value.value());
                }
                Ok(())
            })?;
        }
    }

    struct FieldSpec {
        ident: syn::Ident,
        key: String,
        description: String,
        section: Option<String>,
        default_repr: String,
    }

    let mut parsed_fields = Vec::new();

    for field in &fields_named.named {
        let ident = field.ident.clone().unwrap();
        let mut custom_key: Option<String> = None;
        let mut description = String::new();
        let mut section = default_section.clone();
        let mut default_repr = String::new();

        // Collect doc comments as default description
        let mut doc_lines = Vec::new();
        for attr in &field.attrs {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit_str),
                            ..
                        }),
                    ..
                }) = &attr.meta
                {
                    let val = lit_str.value();
                    let trimmed = val.trim();
                    if !trimmed.is_empty() {
                        doc_lines.push(trimmed.to_string());
                    }
                }
            }
        }
        if !doc_lines.is_empty() {
            description = doc_lines.join(" ");
        }

        // Parse #[setting(...)] attribute
        for attr in &field.attrs {
            if attr.path().is_ident("setting") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("key") {
                        let val: syn::LitStr = meta.value()?.parse()?;
                        custom_key = Some(val.value());
                    } else if meta.path.is_ident("desc") || meta.path.is_ident("description") {
                        let val: syn::LitStr = meta.value()?.parse()?;
                        description = val.value();
                    } else if meta.path.is_ident("section") {
                        let val: syn::LitStr = meta.value()?.parse()?;
                        section = Some(val.value());
                    } else if meta.path.is_ident("default") {
                        let val: syn::LitStr = meta.value()?.parse()?;
                        default_repr = val.value();
                    }
                    Ok(())
                })?;
            }
        }

        let full_key = match custom_key {
            Some(k) => {
                if key_prefix.is_empty() {
                    k
                } else {
                    format!("{key_prefix}{k}")
                }
            }
            None => {
                let name = ident.to_string();
                if key_prefix.is_empty() {
                    name
                } else {
                    format!("{key_prefix}{name}")
                }
            }
        };

        parsed_fields.push(FieldSpec {
            ident,
            key: full_key,
            description,
            section,
            default_repr,
        });
    }

    // 1. Generate schema() metadata vector
    let schema_entries = parsed_fields.iter().map(|f| {
        let key_str = &f.key;
        let desc_str = &f.description;
        let default_str = &f.default_repr;
        let section_expr = match &f.section {
            Some(s) => quote! { ::core::option::Option::Some(#s) },
            None => quote! { ::core::option::Option::None },
        };
        quote! {
            ::goldsrc_api::setting::SettingMeta {
                key: #key_str,
                default_repr: #default_str,
                description: #desc_str,
                section: #section_expr,
            }
        }
    });

    // 2. Generate load_from_tree()
    let load_entries = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let key_str = &f.key;
        quote! {
            if let ::core::option::Option::Some(val_str) = tree.get(#key_str) {
                match val_str.parse() {
                    ::core::result::Result::Ok(parsed) => {
                        let _ = self.#ident.set(parsed)?;
                    }
                    ::core::result::Result::Err(e) => {
                        return ::core::result::Result::Err(::goldsrc_api::setting::SettingError::ParseError {
                            key: #key_str,
                            reason: ::std::format!("failed to parse '{val_str}': {e}"),
                        });
                    }
                }
            }
        }
    });

    // 3. Generate export_tree()
    let export_entries = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let key_str = &f.key;
        quote! {
            tree.insert(
                #key_str.to_string(),
                ::std::format!("{}", self.#ident.get()),
            );
        }
    });

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::goldsrc_api::setting::Settings for #struct_name #ty_generics #where_clause {
            fn schema() -> ::std::vec::Vec<::goldsrc_api::setting::SettingMeta> {
                ::std::vec![
                    #(#schema_entries),*
                ]
            }

            fn load_from_tree(&self, tree: &::goldsrc_api::setting::SettingTree) -> ::core::result::Result<(), ::goldsrc_api::setting::SettingError> {
                #(#load_entries)*
                ::core::result::Result::Ok(())
            }

            fn export_tree(&self) -> ::goldsrc_api::setting::SettingTree {
                let mut tree = ::goldsrc_api::setting::SettingTree::new();
                #(#export_entries)*
                tree
            }
        }
    };

    Ok(expanded)
}
