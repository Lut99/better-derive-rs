//  HASH.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:09:44
//  Last edited:
//    09 Jan 2025, 20:25:42
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Hash`-macro.
//

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens as _, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Data, DeriveInput, Field, Fields, Ident, LitInt, Path, PathArguments, PathSegment, Token, Type, parse_macro_input};

use crate::extract::extract_generics;


/***** HELPER FUNCTIONS *****/
/// Given a list of fields, builds the idents for it and finds their types.
///
/// This resolves both named and unnamed fields to concrete, unique idents.
///
/// # Arguments
/// - `fields`: The [`Fields`] to generate identifiers for.
/// - `use_self`: Whether we're generating for use with `self.` or not (matters for unnamed fields).
///
/// # Returns
/// Two vectors, one with all the generated field identifiers and another with the list of types.
fn generate_field_idents_and_tys(fields: &Punctuated<Field, Token![,]>, use_self: bool) -> (Vec<TokenStream2>, Vec<Type>) {
    let mut fs: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    let mut tys: Vec<Type> = Vec::with_capacity(fields.len());
    for (i, f) in fields.iter().enumerate() {
        // Add the field
        fs.push(f.ident.as_ref().map(Ident::to_token_stream).unwrap_or_else(|| {
            if use_self {
                LitInt::new(&i.to_string(), f.span()).to_token_stream()
            } else {
                Ident::new(&format!("field{i}"), f.span()).to_token_stream()
            }
        }));
        // Add the type
        tys.push(f.ty.clone());
    }
    (fs, tys)
}

/// Builds the necessary hash implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`TokenStream2`] that can be used for the impl.
fn build_hash_impl(input: &DeriveInput) -> TokenStream2 {
    // Match based on the data type
    match &input.data {
        Data::Enum(e) => {
            // Build the impls for every variant
            let mut variants: Vec<TokenStream2> = Vec::with_capacity(e.variants.len());
            for (i, variant) in e.variants.iter().enumerate() {
                let variant_name = &variant.ident;

                // Write depending on the variant form
                variants.push(match &variant.fields {
                    Fields::Named(n) => {
                        let (fields, tys) = generate_field_idents_and_tys(&n.named, false);
                        let impls = fields.iter().zip(tys).map(|(f, t)| quote! { <#t as ::std::hash::Hash>::hash(#f, __state); });
                        quote! {
                            Self::#variant_name { #(#fields),* } => {
                                <usize as ::std::hash::Hash>::hash(&#i, __state);
                                #(#impls)*
                            },
                        }
                    },
                    Fields::Unnamed(u) => {
                        let (fields, tys) = generate_field_idents_and_tys(&u.unnamed, false);
                        let impls = fields.iter().zip(tys).map(|(f, t)| quote! { <#t as ::std::hash::Hash>::hash(#f, __state); });
                        quote! {
                            Self::#variant_name(#(#fields),*) => {
                                <usize as ::std::hash::Hash>::hash(&#i, __state);
                                #(#impls)*
                            },
                        }
                    },
                    Fields::Unit => quote! {
                        Self::#variant_name => {
                            <usize as ::std::hash::Hash>::hash(&#i, __state);
                        },
                    },
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
                let (fields, tys) = generate_field_idents_and_tys(&n.named, true);
                let impls = fields.iter().zip(tys).map(|(f, t)| quote! { <#t as ::std::hash::Hash>::hash(&self.#f, __state); });
                quote! {
                    #(#impls)*
                }
            },
            Fields::Unnamed(u) => {
                let (fields, tys) = generate_field_idents_and_tys(&u.unnamed, true);
                let impls = fields.iter().zip(tys).map(|(f, t)| quote! { <#t as ::std::hash::Hash>::hash(&self.#f, __state); });
                quote! {
                    #(#impls)*
                }
            },
            Fields::Unit => TokenStream2::new(),
        },
        Data::Union(_) => todo!(),
    }
}





/***** LIBRARY *****/
/// Actual implementation of the `Hash` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn hash(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the generics & fmts for the general impl
    let generics = extract_generics(&input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("hash".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Hash".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    });
    let hash = build_hash_impl(&input);

    // Done, build the impl
    let name = &input.ident;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::std::hash::Hash for #name #ty_gen #where_clause {
            #[inline]
            fn hash<H: ::std::hash::Hasher>(&self, __state: &mut H) {
                #hash
            }
        }
    }
    .into()
}
