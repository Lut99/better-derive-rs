//  DEBUG.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:09:44
//  Last edited:
//    05 Feb 2025, 15:42:36
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Debug`-macro.
//

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens as _, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Data, DeriveInput, Field, Fields, Ident, LitInt, Path, PathArguments, PathSegment, Token, parse_macro_input};

use crate::common::{extract_generics, filter_skipped_variants_and_fields};


/***** HELPER FUNCTIONS *****/
/// Given a list of fields, builds the idents for it.
///
/// This resolves both named and unnamed fields to concrete, unique idents.
///
/// # Arguments
/// - `fields`: The [`Fields`] to generate identifiers for.
/// - `use_self`: Whether we're generating for use with `self.` or not (matters for unnamed fields).
///
/// # Returns
/// A vector with all generate field identifiers.
fn generate_field_idents(fields: &Punctuated<Field, Token![,]>, use_self: bool) -> Vec<TokenStream2> {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            f.ident.as_ref().map(Ident::to_token_stream).unwrap_or_else(|| {
                if use_self {
                    LitInt::new(&i.to_string(), f.span()).to_token_stream()
                } else {
                    Ident::new(&format!("field{i}"), f.span()).to_token_stream()
                }
            })
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
fn build_fmt_impl(input: &DeriveInput) -> TokenStream2 {
    let sname = input.ident.to_string();

    // Match based on the data type
    match &input.data {
        Data::Enum(e) => {
            // Build the impls for every variant
            let mut variants: Vec<TokenStream2> = Vec::with_capacity(e.variants.len());
            for variant in &e.variants {
                let variant_name = &variant.ident;
                let svariant_name = variant_name.to_string();

                // Write depending on the variant form
                variants.push(match &variant.fields {
                    Fields::Named(n) => {
                        let fields = generate_field_idents(&n.named, false);
                        quote! {
                            Self::#variant_name { #(#fields),* } => {
                                let mut __fmt = __f.debug_struct(::std::concat!(#sname, "::", #svariant_name));
                                #(
                                    __fmt.field(::std::stringify!(#fields), #fields);
                                )*
                                __fmt.finish()
                            },
                        }
                    },
                    Fields::Unnamed(u) => {
                        let fields = generate_field_idents(&u.unnamed, false);
                        quote! {
                            Self::#variant_name(#(#fields),*) => {
                                let mut __fmt = __f.debug_tuple(::std::concat!(#sname, "::", #svariant_name));
                                #(
                                    __fmt.field(#fields);
                                )*
                                __fmt.finish()
                            },
                        }
                    },
                    Fields::Unit => quote! { Self::#variant_name => ::std::write!(__f, ::std::concat!(#sname, "::", #svariant_name)), },
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
                let fields = generate_field_idents(&n.named, true);
                quote! {
                    let mut __fmt = __f.debug_struct(#sname);
                    #(
                        __fmt.field(::std::stringify!(#fields), &self.#fields);
                    )*
                    __fmt.finish()
                }
            },
            Fields::Unnamed(u) => {
                let fields = generate_field_idents(&u.unnamed, true);
                quote! {
                    let mut __fmt = __f.debug_tuple(#sname);
                    #(
                        __fmt.field(&self.#fields);
                    )*
                    __fmt.finish()
                }
            },
            Fields::Unit => quote! { ::std::write!(__f, #sname) },
        },
        Data::Union(_) => todo!(),
    }
}




/***** LIBRARY *****/
/// Actual implementation of the `Debug` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn debug(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    // Filter the input data
    if let Err(err) = filter_skipped_variants_and_fields("debug", &mut input.data) {
        return err.into_compile_error().into();
    }

    // Extract the generics & fmts for the general impl
    let generics = extract_generics(&input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("fmt".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Debug".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    });
    let fmt = build_fmt_impl(&input);

    // Done, build the impl
    let name = &input.ident;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::std::fmt::Debug for #name #ty_gen #where_clause {
            #[inline]
            fn fmt(&self, __f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #fmt
            }
        }
    }
    .into()
}
