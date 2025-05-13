//  SERIALIZE.rs
//    by Lut99
//
//  Description:
//!   Implements the `Serialize`-macro.
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
/// A vector with all the generated field identifiers.
fn generate_field_idents(fields: &Punctuated<Field, Token![,]>, use_self: bool) -> Vec<TokenStream2> {
    let mut fs: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    for (i, f) in fields.iter().enumerate() {
        // Add the field
        fs.push(f.ident.as_ref().map(Ident::to_token_stream).unwrap_or_else(|| {
            if use_self {
                LitInt::new(&i.to_string(), f.span()).to_token_stream()
            } else {
                Ident::new(&format!("field{i}"), f.span()).to_token_stream()
            }
        }));
    }
    fs
}

/// Builds the necessary hash implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
///
/// # Returns
/// A [`TokenStream2`] that can be used for the impl.
fn build_serde_impl(input: &DeriveInput) -> TokenStream2 {
    // Match based on the data type
    let name: &Ident = &input.ident;
    match &input.data {
        Data::Enum(e) => {
            // Build the impls for every variant
            let mut variants: Vec<TokenStream2> = Vec::with_capacity(e.variants.len());
            for (i, variant) in e.variants.iter().enumerate() {
                let i_32: u32 = i as u32;
                let variant_name = &variant.ident;

                // Write depending on the variant form
                variants.push(match &variant.fields {
                    Fields::Named(n) => {
                        let fields = generate_field_idents(&n.named, false);
                        let fields_len: usize = fields.len();
                        let impls = fields.iter().map(|f| quote! { <<SE as ::serde::Serializer>::SerializeStructVariant as ::serde::ser::SerializeStructVariant>::serialize_field(&mut __ser, ::std::stringify!(#f), #f)?; });
                        quote! {
                            Self::#variant_name { #(#fields),* } => {
                                let mut __ser = <SE as ::serde::Serializer>::serialize_struct_variant(__serializer, ::std::stringify!(#name), #i_32, ::std::stringify!(#variant_name), #fields_len)?;
                                #(#impls)*
                                <<SE as ::serde::Serializer>::SerializeStructVariant as ::serde::ser::SerializeStructVariant>::end(__ser)
                            },
                        }
                    },
                    Fields::Unnamed(u) => {
                        let fields = generate_field_idents(&u.unnamed, false);
                        let fields_len: usize = fields.len();
                        if fields_len == 1 {
                            // We serialize as a newtype instead
                            let f = fields.first().unwrap();
                            quote! {
                                Self::#variant_name(#(#fields),*) => {
                                    <SE as ::serde::Serializer>::serialize_newtype_variant(__serializer, ::std::stringify!(#name), #i_32, ::std::stringify!(#variant_name), #f)
                                },
                            }
                        } else {
                            let impls = fields.iter().map(|f| quote! { <<SE as ::serde::Serializer>::SerializeTupleVariant as ::serde::ser::SerializeTupleVariant>::serialize_field(&mut __ser, #f)?; });
                            quote! {
                                Self::#variant_name(#(#fields),*) => {
                                    let mut __ser = <SE as ::serde::Serializer>::serialize_tuple_variant(__serializer, ::std::stringify!(#name), #i_32, ::std::stringify!(#variant_name), #fields_len)?;
                                    #(#impls)*
                                    <<SE as ::serde::Serializer>::SerializeTupleVariant as ::serde::ser::SerializeTupleVariant>::end(__ser)
                                },
                            }
                        }
                    },
                    Fields::Unit => quote! {
                        Self::#variant_name => {
                            <SE as ::serde::Serializer>::serialize_unit_variant(__serializer, ::std::stringify!(#name), #i_32, ::std::stringify!(#variant_name))
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
                let fields = generate_field_idents(&n.named, true);
                let fields_len: usize = fields.len();
                let impls = fields
                    .iter()
                    .map(|f| quote! { <<SE as ::serde::Serializer>::SerializeStruct as ::serde::ser::SerializeStruct>::serialize_field(&mut __ser, ::std::stringify!(#f), &self.#f)?; });
                quote! {
                    let mut __ser = <SE as ::serde::Serializer>::serialize_struct(__serializer, ::std::stringify!(#name), #fields_len)?;
                    #(#impls)*
                    <<SE as ::serde::Serializer>::SerializeStruct as ::serde::ser::SerializeStruct>::end(__ser)
                }
            },
            Fields::Unnamed(u) => {
                let fields = generate_field_idents(&u.unnamed, true);
                let fields_len: usize = fields.len();
                if fields_len == 1 {
                    // We serialize as a newtype instead
                    let f = fields.first().unwrap();
                    quote! {
                        <SE as ::serde::Serializer>::serialize_newtype_struct(__serializer, ::std::stringify!(#name), &self.#f)
                    }
                } else {
                    let impls = fields.iter().map(|f| quote! { <<SE as ::serde::Serializer>::SerializeTupleStruct as ::serde::ser::SerializeTupleStruct>::serialize_field(&mut __ser, &self.#f)?; });
                    quote! {
                        let mut __ser = <SE as ::serde::Serializer>::serialize_tuple_struct(__serializer, ::std::stringify!(#name), #fields_len)?;
                        #(#impls)*
                        <<SE as ::serde::Serializer>::SerializeTupleStruct as ::serde::ser::SerializeTupleStruct>::end(__ser)
                    }
                }
            },
            Fields::Unit => quote! {
                <SE as ::serde::Serializer>::serialize_unit_struct(__serializer, ::std::stringify!(#name))
            },
        },
        Data::Union(_) => todo!(),
    }
}





/***** LIBRARY *****/
/// Actual implementation of the `Serialize` derive macro.
///
/// # Arguments
/// - `input`: The [`TokenStream2`] describing the data container to derive for.
///
/// # Returns
/// A [`TokenSream2`] encoding the impl.
pub fn serialize(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    // Filter the input data
    if let Err(err) = filter_skipped_variants_and_fields("serialize", &mut input.data) {
        return err.into_compile_error().into();
    }

    // Extract the generics & fmts for the general impl
    let generics = match extract_generics("serialize", &input.attrs, &input, &Path {
        leading_colon: Some(Default::default()),
        segments:      {
            let mut segments = Punctuated::new();
            segments.push(PathSegment { ident: Ident::new("serde".into(), Span::call_site()), arguments: PathArguments::None });
            segments.push(PathSegment { ident: Ident::new("Serialize".into(), Span::call_site()), arguments: PathArguments::None });
            segments
        },
    }) {
        Ok(gens) => gens,
        Err(err) => return err.into_compile_error().into(),
    };
    let serde = build_serde_impl(&input);

    // Done, build the impl
    let name = &input.ident;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_gen ::serde::Serialize for #name #ty_gen #where_clause {
            fn serialize<SE>(&self, __serializer: SE) -> ::std::result::Result<SE::Ok, SE::Error>
            where
                SE: ::serde::Serializer,
            {
                #serde
            }
        }
    }
    .into()
}
