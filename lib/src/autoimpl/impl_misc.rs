// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Miscellaneous impls

use super::{ImplArgs, ImplTrait, Result};
use crate::SimplePath;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{Fields, Index, ItemStruct};

/// Implement [`core::clone::Clone`]
pub struct ImplClone;
impl ImplTrait for ImplClone {
    fn path(&self) -> SimplePath {
        SimplePath::new(&["", "core", "clone", "Clone"])
    }

    fn support_ignore(&self) -> bool {
        true
    }

    fn support_using(&self) -> bool {
        false
    }

    fn struct_items(&self, item: &ItemStruct, args: &ImplArgs) -> Result<TokenStream> {
        let type_ident = &item.ident;
        let inner = match &item.fields {
            Fields::Named(fields) => {
                let mut toks = TokenStream::new();
                for field in fields.named.iter() {
                    let ident = field.ident.as_ref().unwrap();
                    if args.ignore_named(ident) {
                        toks.append_all(quote! { #ident: Default::default(), });
                    } else {
                        toks.append_all(quote! { #ident: self.#ident.clone(), });
                    }
                }
                quote! { #type_ident { #toks } }
            }
            Fields::Unnamed(fields) => {
                let mut toks = TokenStream::new();
                for i in 0..fields.unnamed.len() {
                    let index = Index::from(i);
                    if args.ignore_unnamed(&index) {
                        toks.append_all(quote! { Default::default(), });
                    } else {
                        toks.append_all(quote! { self.#index.clone(), });
                    }
                }
                quote! { #type_ident ( #toks ) }
            }
            Fields::Unit => quote! { #type_ident },
        };
        Ok(quote! {
            fn clone(&self) -> Self {
                #inner
            }
        })
    }
}

/// Implement [`core::fmt::Debug`]
pub struct ImplDebug;
impl ImplTrait for ImplDebug {
    fn path(&self) -> SimplePath {
        SimplePath::new(&["", "core", "fmt", "Debug"])
    }

    fn support_ignore(&self) -> bool {
        true
    }

    fn support_using(&self) -> bool {
        false
    }

    fn struct_items(&self, item: &ItemStruct, args: &ImplArgs) -> Result<TokenStream> {
        let type_name = item.ident.to_string();
        let mut inner;
        match &item.fields {
            Fields::Named(fields) => {
                inner = quote! { f.debug_struct(#type_name) };
                let mut no_skips = true;
                for field in fields.named.iter() {
                    let ident = field.ident.as_ref().unwrap();
                    if !args.ignore_named(ident) {
                        let name = ident.to_string();
                        inner.append_all(quote! {
                            .field(#name, &self.#ident)
                        });
                    } else {
                        no_skips = false;
                    }
                }
                if no_skips {
                    inner.append_all(quote! { .finish() });
                } else {
                    inner.append_all(quote! { .finish_non_exhaustive() });
                };
            }
            Fields::Unnamed(fields) => {
                inner = quote! { f.debug_tuple(#type_name) };
                for i in 0..fields.unnamed.len() {
                    let index = Index::from(i);
                    if !args.ignore_unnamed(&index) {
                        inner.append_all(quote! {
                            .field(&self.#index)
                        });
                    } else {
                        inner.append_all(quote! {
                            .field(&format_args!("_"))
                        });
                    }
                }
                inner.append_all(quote! { .finish() });
            }
            Fields::Unit => inner = quote! { f.write_str(#type_name) },
        };
        Ok(quote! {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                #inner
            }
        })
    }
}

/// Implement [`core::default::Default`]
pub struct ImplDefault;
impl ImplTrait for ImplDefault {
    fn path(&self) -> SimplePath {
        SimplePath::new(&["", "core", "default", "Default"])
    }

    fn support_ignore(&self) -> bool {
        false
    }

    fn support_using(&self) -> bool {
        false
    }

    fn struct_items(&self, item: &ItemStruct, _: &ImplArgs) -> Result<TokenStream> {
        let type_ident = &item.ident;
        let mut inner;
        match &item.fields {
            Fields::Named(fields) => {
                inner = quote! {};
                for field in fields.named.iter() {
                    let ident = field.ident.as_ref().unwrap();
                    inner.append_all(quote! { #ident: Default::default(), });
                }
                inner = quote! { #type_ident { #inner } };
            }
            Fields::Unnamed(fields) => {
                inner = quote! {};
                for _ in 0..fields.unnamed.len() {
                    inner.append_all(quote! { Default::default(), });
                }
                inner = quote! { #type_ident(#inner) };
            }
            Fields::Unit => inner = quote! { #type_ident },
        }
        Ok(quote! {
            fn default() -> Self {
                #inner
            }
        })
    }
}
