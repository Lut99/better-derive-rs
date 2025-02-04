//  CLONE.rs
//    by Lut99
//
//  Created:
//    04 Feb 2025, 15:41:18
//  Last edited:
//    04 Feb 2025, 16:33:51
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Clone`-macro.
//

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens as _, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Data, DeriveInput, Field, Fields, Ident, LitInt, Path, PathArguments, PathSegment, Token, Type, parse_macro_input};

use crate::extract::extract_generics;


/***** HELPER FUNCTIONS *****/
/// Given a list of fields, builds the idents for it and collects their types.
///
/// This resolves both named and unnamed fields to concrete, unique idents.
///
/// # Arguments
/// - `fields`: The [`Fields`] to generate identifiers for.
/// - `use_self`: Whether we're generating for use with `self.` or not (matters for unnamed fields).
///
/// # Returns
/// A vector with tuples of all generated field identifiers and their matching types.
fn generate_field_idents_and_tys(fields: &Punctuated<Field, Token![,]>, use_self: bool) -> Vec<(TokenStream2, Type)> {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            (
                f.ident.as_ref().map(Ident::to_token_stream).unwrap_or_else(|| {
                    if use_self {
                        LitInt::new(&i.to_string(), f.span()).to_token_stream()
                    } else {
                        Ident::new(&format!("field{i}"), f.span()).to_token_stream()
                    }
                }),
                f.ty.clone(),
            )
        })
        .collect()
}

/// Builds the necessary formatter implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`TokenStream2`] that can be used for the impl.
fn build_clone_impl(input: &DeriveInput) -> TokenStream2 {
    // Match based on the data type
    match &input.data {
        Data::Enum(e) => {
            // Build the impls for every variant
            let mut variants: Vec<TokenStream2> = Vec::with_capacity(e.variants.len());
            for variant in &e.variants {
                let variant_name = &variant.ident;

                // Write depending on the variant form
                variants.push(match &variant.fields {
                    Fields::Named(n) => {
                        let fields_tys = generate_field_idents_and_tys(&n.named, false);
                        let fields: Vec<&TokenStream2> = fields_tys.iter().map(|(f, _)| f).collect();
                        let clone_fields: Vec<TokenStream2> =
                            fields_tys.iter().map(|(f, t)| quote! { #f: <#t as ::std::clone::Clone>::clone(#f) }).collect();
                        quote! {
                            Self::#variant_name { #(#fields),* } => {
                                Self::#variant_name { #(#clone_fields),* }
                            },
                        }
                    },
                    Fields::Unnamed(u) => {
                        let fields_tys = generate_field_idents_and_tys(&u.unnamed, false);
                        let fields: Vec<&TokenStream2> = fields_tys.iter().map(|(f, _)| f).collect();
                        let clone_fields: Vec<TokenStream2> =
                            fields_tys.iter().map(|(f, t)| quote! { <#t as ::std::clone::Clone>::clone(#f) }).collect();
                        quote! {
                            Self::#variant_name(#(#fields),*) => {
                                Self::#variant_name(#(#clone_fields),*)
                            },
                        }
                    },
                    Fields::Unit => quote! { Self::#variant_name => Self::#variant_name, },
                });
            }

            // Build the full match
            if !variants.is_empty() {
                quote! {
                    match self {
                        #(#variants)*
                    }
                }
            } else {
                quote! { ::std::unreachable!() }
            }
        },
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => {
                let fields_tys = generate_field_idents_and_tys(&n.named, true);
                let clone_fields: Vec<TokenStream2> =
                    fields_tys.iter().map(|(f, t)| quote! { #f: <#t as ::std::clone::Clone>::clone(&self.#f) }).collect();
                quote! {
                    Self { #(#clone_fields),* }
                }
            },
            Fields::Unnamed(u) => {
                let fields_tys = generate_field_idents_and_tys(&u.unnamed, true);
                let clone_fields: Vec<TokenStream2> =
                    fields_tys.iter().map(|(f, t)| quote! { <#t as ::std::clone::Clone>::clone(&self.#f) }).collect();
                quote! {
                    Self(#(#clone_fields),*)
                }
            },
            Fields::Unit => quote! { Self },
        },
        Data::Union(_) => todo!(),
    }
}





/***** LIBRARY *****/
/// Actual implementation of the `Clone` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn clone(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the generics & fmts for the general impl
    let generics = extract_generics(&input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("clone".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Clone".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    });
    let clone = build_clone_impl(&input);

    // Done, build the impl
    let name = &input.ident;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::std::clone::Clone for #name #ty_gen #where_clause {
            #[inline]
            fn clone(&self) -> Self {
                #clone
            }
        }
    }
    .into()
}
