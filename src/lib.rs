//  LIB.rs
//    by Lut99
//
//  Created:
//    26 Dec 2024, 11:47:57
//  Last edited:
//    26 Dec 2024, 13:03:03
//  Auto updated?
//    Yes
//
//  Description:
//!   <Todo>
//

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{
    Data, DeriveInput, Fields, Generics, Ident, LitInt, Path, PathArguments, PathSegment, PredicateType, TraitBound, TraitBoundModifier, Type,
    TypeParamBound, WhereClause, WherePredicate, parse_macro_input,
};


/***** HELPER FUNCTIONS *****/
/// Collects the generics necessary for the [`Debug`](std::fmt::Debug) implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`Generics`] that can be used for the impl.
fn extract_generics(input: &DeriveInput) -> Generics {
    // Simply find a list of all types in the struct
    let mut generics = input.generics.clone();
    let mut where_clause =
        generics.where_clause.clone().unwrap_or_else(|| WhereClause { where_token: Default::default(), predicates: Punctuated::new() });
    for ty in match &input.data {
        Data::Enum(e) => Box::new(e.variants.iter().flat_map(|variant| match &variant.fields {
            Fields::Named(n) => Box::new(n.named.iter().map(|f| &f.ty)) as Box<dyn Iterator<Item = &Type>>,
            Fields::Unnamed(u) => Box::new(u.unnamed.iter().map(|f| &f.ty)),
            Fields::Unit => Box::new(None::<&Type>.into_iter()),
        })) as Box<dyn Iterator<Item = &Type>>,
        Data::Struct(s) => Box::new(match &s.fields {
            Fields::Named(n) => Box::new(n.named.iter().map(|f| &f.ty)) as Box<dyn Iterator<Item = &Type>>,
            Fields::Unnamed(u) => Box::new(u.unnamed.iter().map(|f| &f.ty)),
            Fields::Unit => Box::new(None::<&Type>.into_iter()),
        }),
        Data::Union(_) => todo!(),
    } {
        where_clause.predicates.push(WherePredicate::Type(PredicateType {
            lifetimes:   None,
            bounded_ty:  ty.clone(),
            colon_token: Default::default(),
            bounds:      {
                let mut bounds = Punctuated::new();
                bounds.push(TypeParamBound::Trait(TraitBound {
                    paren_token: Default::default(),
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: Some(Default::default()),
                        segments:      {
                            let mut segments = Punctuated::new();
                            segments.push(PathSegment { ident: Ident::new("std".into(), Span::call_site()), arguments: PathArguments::None });
                            segments.push(PathSegment { ident: Ident::new("fmt".into(), Span::call_site()), arguments: PathArguments::None });
                            segments.push(PathSegment { ident: Ident::new("Debug".into(), Span::call_site()), arguments: PathArguments::None });
                            segments
                        },
                    },
                }));
                bounds
            },
        }));
    }
    generics.where_clause = Some(where_clause);
    generics
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
                        let fields: Vec<Ident> = n
                            .named
                            .iter()
                            .enumerate()
                            .map(|(i, f)| f.ident.clone().unwrap_or_else(|| Ident::new(&format!("field{i}"), f.span())))
                            .collect();
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
                        let fields: Vec<Ident> = u
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, f)| f.ident.clone().unwrap_or_else(|| Ident::new(&format!("field{i}"), f.span())))
                            .collect();
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
                let fields: Vec<Ident> =
                    n.named.iter().enumerate().map(|(i, f)| f.ident.clone().unwrap_or_else(|| Ident::new(&format!("field{i}"), f.span()))).collect();
                quote! {
                    let mut __fmt = __f.debug_struct(#sname);
                    #(
                        __fmt.field(::std::stringify!(#fields), &self.#fields);
                    )*
                    __fmt.finish()
                }
            },
            Fields::Unnamed(u) => {
                let fields: Vec<LitInt> = u.unnamed.iter().enumerate().map(|(i, f)| LitInt::new(&i.to_string(), f.span())).collect();
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





/***** MACROS *****/
/// Defines a [`Debug`](::std::Debug)-like derive macro that's more lenient to generics.
///
/// In particular, the default derive macro enforces that all _generics_ implement
/// [`Debug`](std::fmt::Debug). This is, however, too strict. Instead, all that's needed is that
/// the _fields_ implement it, which may or may not require the generics to do so.
///
/// You can use this macro in exactly the same way as the builtin one.
///
/// # Examples
/// ```rust
/// use std::marker::PhantomData;
///
/// use debug::Debug;
///
/// struct DebuglessType;
///
/// #[derive(Debug)]
/// struct PhantomStruct<T> {
///     _t: PhantomData<T>,
/// }
///
/// assert_eq!(
///     format!("{:?}", PhantomStruct::<DebuglessType> { _t: PhantomData }),
///     "PhantomStruct { _t: \
///      PhantomData<rust_out::main::_doctest_main_src_lib_rs_193_0::DebuglessType> }"
/// )
/// ```
#[proc_macro_derive(Debug)]
pub fn debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the generics & fmts for the general impl
    let generics = extract_generics(&input);
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
