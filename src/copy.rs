//  COPY.rs
//    by Lut99
//
//  Created:
//    04 Feb 2025, 16:33:42
//  Last edited:
//    06 Feb 2025, 10:34:18
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Copy`-macro.
//


use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, Path, PathArguments, PathSegment, parse_macro_input};

use crate::common::extract_generics;


/***** LIBRARY *****/
/// Actual implementation of the `Copy` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn copy(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the generics & fmts for the general impl
    let (impl_gen, ty_gen, where_clause) = match extract_generics("copy", &input.attrs, &input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("marker".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Copy".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    }) {
        Ok(gens) => gens,
        Err(err) => return err.into_compile_error().into(),
    };

    // Done, build the impl
    let name = &input.ident;
    quote! {
        impl #impl_gen ::std::marker::Copy for #name #ty_gen #where_clause {}
    }
    .into()
}
