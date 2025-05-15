//  PARTIAL ORD.rs
//    by Lut99
//
//  Created:
//    13 Feb 2025, 11:24:57
//  Last edited:
//    13 Feb 2025, 15:34:01
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Ord`-macro.
//


use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, Path, PathArguments, PathSegment, parse_macro_input};

use crate::common::extract_generics;


/***** HELPER FUNCTIONS *****/
/// Builds the necessary ordering implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`TokenStream2`] that can be used for the impl.
fn build_cmp_impl() -> TokenStream2 {
    // We can actually rely on the partial_cmp impl for this one!
    quote! { <Self as ::std::cmp::PartialOrd>::partial_cmp(self, __other).unwrap_or_else(|| panic!("<{} as ::std::cmp::PartialOrd>::partial_cmp() is not fully implemented (i.e., the type is not fully ordered at all!)", ::std::any::type_name::<Self>())) }
}





/***** LIBRARY *****/
/// Actual implementation of the `Ord` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn ord(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the generics & fmts for the general impl
    let (impl_gen, ty_gen, where_clause) = match extract_generics("ord", &input.attrs, &input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("cmp".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Ord".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    }) {
        Ok(gens) => gens,
        Err(err) => return err.into_compile_error().into(),
    };
    let cmp = build_cmp_impl();

    // Done, build the impl
    let name = &input.ident;
    quote! {
        impl #impl_gen ::std::cmp::Ord for #name #ty_gen #where_clause {
            #[inline]
            fn cmp(&self, __other: &Self) -> ::std::cmp::Ordering {
                #cmp
            }
        }
    }
    .into()
}
