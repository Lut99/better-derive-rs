//  EQ.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:17:54
//  Last edited:
//    06 Feb 2025, 10:34:31
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Eq`-macro.
//


use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, Path, PathArguments, PathSegment, parse_macro_input};

use crate::common::extract_generics;


/***** LIBRARY *****/
/// Actual implementation of the `Eq` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn eq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the generics & fmts for the general impl
    let generics = match extract_generics("eq", &input.attrs, &input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("cmp".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Eq".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    }) {
        Ok(gens) => gens,
        Err(err) => return err.into_compile_error().into(),
    };

    // Done, build the impl
    let name = &input.ident;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::std::cmp::Eq for #name #ty_gen #where_clause {}
    }
    .into()
}
