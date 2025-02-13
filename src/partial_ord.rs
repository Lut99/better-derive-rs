//  PARTIAL ORD.rs
//    by Lut99
//
//  Created:
//    13 Feb 2025, 11:24:57
//  Last edited:
//    13 Feb 2025, 15:28:15
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the `PartialOrd`-macro.
//


use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{
    Data, DeriveInput, Error, Expr, ExprLit, Field, Fields, Ident, Lit, LitInt, Path, PathArguments, PathSegment, Token, Type, parse_macro_input,
};

use crate::common::{extract_generics, filter_skipped_variants_and_fields};


/***** HELPER FUNCTIONS *****/
/// Given a list of fields, builds the left idents, right idents and ordering pairs.
///
/// # Arguments
/// - `fields`: The [`Fields`] to generate identifiers for.
/// - `use_self`: Whether to inject `self.` in the ordering pairs.
///
/// # Returns
/// Three vectors that encode the list of left field identifiers, right field identifiers and
/// ordering pairs.
fn generate_field_idents(fields: &Punctuated<Field, Token![,]>, use_self: bool) -> (Vec<TokenStream2>, Vec<TokenStream2>, Vec<TokenStream2>) {
    let mut ls: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    let mut rs: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    let mut partial_ord: Vec<TokenStream2> = Vec::with_capacity(fields.len());
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
            partial_ord.push(quote! { <#ty as ::std::cmp::PartialOrd>::partial_cmp(&self.#lident, &__other.#rident) });
        } else {
            partial_ord.push(quote! { <#ty as ::std::cmp::PartialOrd>::partial_cmp(#lident, #rident) });
        }
    }
    (ls, rs, partial_ord)
}

/// Builds the necessary ordering implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`TokenStream2`] that can be used for the impl.
fn build_partial_cmp_impl(input: &DeriveInput) -> Result<TokenStream2, Error> {
    // Match based on the data type
    match &input.data {
        Data::Enum(e) => {
            // Build the impls for every variant
            let mut discriminants: Vec<(isize, TokenStream2)> = Vec::with_capacity(e.variants.len());
            let mut variants: Vec<TokenStream2> = Vec::with_capacity(e.variants.len());
            for variant in &e.variants {
                let variant_name = &variant.ident;

                // First we create a match pattern for finding the variant's discriminator
                let prev_discriminant: isize = discriminants.last().map(|(d, _)| *d).unwrap_or(0);
                let discriminant: isize = variant
                    .discriminant
                    .as_ref()
                    .map(|(_, expr)| match expr {
                        Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) => i.base10_parse(),
                        expr => Err(Error::new(expr.span(), "Expected an integer literal")),
                    })
                    .transpose()?
                    .unwrap_or(prev_discriminant + 1);
                discriminants.push((discriminant, quote! { Self::#variant_name { .. } => #discriminant, }));

                // Write depending on the variant form
                variants.push(match &variant.fields {
                    Fields::Named(n) => {
                        // Generate field names for both halfs
                        let (lfields, rfields, eqfields) = generate_field_idents(&n.named, false);

                        // Now generate
                        quote! {
                            (Self::#variant_name { #(#lfields),* }, Self::#variant_name { #(#rfields),* }) => {
                                #(match #eqfields {
                                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {},
                                    ord => return ord,
                                })*
                                ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                            },
                        }
                    },
                    Fields::Unnamed(u) => {
                        // Generate field names for both halfs
                        let (lfields, rfields, eqfields) = generate_field_idents(&u.unnamed, false);

                        // Now generate
                        quote! {
                            (Self::#variant_name(#(#lfields),*), Self::#variant_name(#(#rfields),*)) => {
                                #(match #eqfields {
                                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {},
                                    ord => return ord,
                                })*
                                ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                            },
                        }
                    },
                    Fields::Unit => {
                        quote! { (Self::#variant_name, Self::#variant_name) => ::std::option::Option::Some(::std::cmp::Ordering::Equal), }
                    },
                });
            }
            let discriminants: Vec<TokenStream2> = discriminants.into_iter().map(|(_, t)| t).collect();

            // Build the full match
            if !variants.is_empty() {
                Ok(quote! {
                    match (self, __other) {
                        #(#variants)*
                        (this, other) => match this { #(#discriminants)* }.partial_cmp(&match other { #(#discriminants)* }),
                    }
                })
            } else {
                Ok(quote! { ::std::unreachable!() })
            }
        },
        Data::Struct(s) => match &s.fields {
            Fields::Named(n) => {
                let (_, _, eqfields) = generate_field_idents(&n.named, true);
                Ok(quote! {
                    #(match #eqfields {
                        ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {},
                        ord => return ord,
                    })*
                    ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                })
            },
            Fields::Unnamed(u) => {
                let (_, _, eqfields) = generate_field_idents(&u.unnamed, true);
                Ok(quote! {
                    #(match #eqfields {
                        ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {},
                        ord => return ord,
                    })*
                    ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                })
            },
            Fields::Unit => Ok(quote! { ::std::option::Option::Some(::std::cmp::Ordering::Equal) }),
        },
        Data::Union(_) => todo!(),
    }
}





/***** LIBRARY *****/
/// Actual implementation of the `PartialOrd` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn partial_ord(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    // Filter the input data
    if let Err(err) = filter_skipped_variants_and_fields("partial_ord", &mut input.data) {
        return err.into_compile_error().into();
    }

    // Extract the generics & fmts for the general impl
    let generics = match extract_generics("partial_ord", &input.attrs, &input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("cmp".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("PartialOrd".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    }) {
        Ok(gens) => gens,
        Err(err) => return err.into_compile_error().into(),
    };
    let partial_cmp = match build_partial_cmp_impl(&input) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error().into(),
    };

    // Done, build the impl
    let name = &input.ident;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::std::cmp::PartialOrd for #name #ty_gen #where_clause {
            #[inline]
            fn partial_cmp(&self, __other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                #partial_cmp
            }
        }
    }
    .into()
}
