//  PARTIAL EQ.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:27:30
//  Last edited:
//    06 Feb 2025, 15:44:56
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `Eq`-macro.
//


use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{Data, DeriveInput, Field, Fields, Ident, LitInt, Path, PathArguments, PathSegment, Token, Type, parse_macro_input};

use crate::common::{extract_generics, filter_skipped_variants_and_fields};


/***** HELPER FUNCTIONS *****/
/// Given a list of fields, builds the left idents, right idents and eq pairs.
///
/// # Arguments
/// - `fields`: The [`Fields`] to generate identifiers for.
/// - `use_self`: Whether to inject `self.` in the eq pairs.
///
/// # Returns
/// Three vectors that encode the list of left field identifiers, right field identifiers and eq
/// pairs.
fn generate_field_idents(fields: &Punctuated<Field, Token![,]>, use_self: bool) -> (Vec<TokenStream2>, Vec<TokenStream2>, Vec<TokenStream2>) {
    let mut ls: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    let mut rs: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    let mut eq: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    for (i, f) in fields.iter().enumerate() {
        // Generate the common identifier(s)
        let (ident, span): (String, Span) = f
            .ident
            .as_ref()
            .map(|i| (i.to_string(), i.span()))
            .unwrap_or_else(|| (if use_self { i.to_string() } else { format!("field{i}") }, f.span()));

        // Generate the left- and right versions
        let (lident, rident): (TokenStream2, TokenStream2) = match (use_self, f.ident.is_some()) {
            (true, true) => {
                let ident = Ident::new(&ident, span).to_token_stream();
                ls.push(ident.clone());
                rs.push(ident.clone());
                (ident.clone(), ident)
            },
            (true, false) => {
                let ident = LitInt::new(&ident, span).to_token_stream();
                ls.push(ident.clone());
                rs.push(ident.clone());
                (ident.clone(), ident)
            },
            (false, true) => {
                let ident = Ident::new(&ident, span);
                let left = Ident::new(&format!("l{ident}"), span);
                let right = Ident::new(&format!("r{ident}"), span);
                ls.push(quote! { #ident: #left });
                rs.push(quote! { #ident: #right });
                (left.to_token_stream(), right.to_token_stream())
            },
            (false, false) => {
                let lident = Ident::new(&format!("l{ident}"), span).to_token_stream();
                let rident = Ident::new(&format!("r{ident}"), span).to_token_stream();
                ls.push(lident.clone());
                rs.push(rident.clone());
                (lident, rident)
            },
        };

        // Inject into the lists
        let ty: &Type = &f.ty;
        if use_self {
            eq.push(quote! { <#ty as ::std::cmp::PartialEq>::eq(&self.#lident, &__other.#rident) });
        } else {
            eq.push(quote! { <#ty as ::std::cmp::PartialEq>::eq(#lident, #rident) });
        }
    }
    (ls, rs, eq)
}

/// Builds the necessary equality implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`TokenStream2`] that can be used for the impl.
fn build_eq_impl(input: &DeriveInput) -> TokenStream2 {
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
                        // Generate field names for both halfs
                        let (lfields, rfields, eqfields) = generate_field_idents(&n.named, false);

                        // Now generate
                        quote! {
                            (Self::#variant_name { #(#lfields),* }, Self::#variant_name { #(#rfields),* }) => {
                                true #(&& #eqfields)*
                            },
                        }
                    },
                    Fields::Unnamed(u) => {
                        // Generate field names for both halfs
                        let (lfields, rfields, eqfields) = generate_field_idents(&u.unnamed, false);

                        // Now generate
                        quote! {
                            (Self::#variant_name(#(#lfields),*), Self::#variant_name(#(#rfields),*)) => {
                                true #(&& #eqfields)*
                            },
                        }
                    },
                    Fields::Unit => quote! { (Self::#variant_name, Self::#variant_name) => true, },
                });
            }

            // Build the full match
            if !variants.is_empty() {
                quote! {
                    match (self, __other) {
                        #(#variants)*
                        _ => false,
                    }
                }
            } else {
                quote! { ::std::unreachable!() }
            }
        },
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => {
                let (_, _, eqfields) = generate_field_idents(&n.named, true);
                quote! {
                    true #(&& #eqfields)*
                }
            },
            Fields::Unnamed(u) => {
                let (_, _, eqfields) = generate_field_idents(&u.unnamed, true);
                quote! {
                    true #(&& #eqfields)*
                }
            },
            Fields::Unit => quote! { true },
        },
        Data::Union(_) => todo!(),
    }
}





/***** LIBRARY *****/
/// Actual implementation of the `PartialEq` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn partial_eq(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    // Filter the input data
    if let Err(err) = filter_skipped_variants_and_fields("partial_eq", &mut input.data) {
        return err.into_compile_error().into();
    }

    // Extract the generics & fmts for the general impl
    let (impl_gen, ty_gen, where_clause) = match extract_generics("partial_eq", &input.attrs, &input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("cmp".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("PartialEq".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    }) {
        Ok(gens) => gens,
        Err(err) => return err.into_compile_error().into(),
    };
    let eq = build_eq_impl(&input);

    // Done, build the impl
    let name = &input.ident;
    quote! {
        impl #impl_gen ::std::cmp::PartialEq for #name #ty_gen #where_clause {
            #[inline]
            fn eq(&self, __other: &Self) -> bool {
                #eq
            }
        }
    }
    .into()
}
