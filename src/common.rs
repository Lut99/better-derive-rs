//  COMMON.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:10:02
//  Last edited:
//    05 Feb 2025, 15:58:33
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines common functionality between the macros.
//

use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::token::Comma;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, Generics, Meta, Path, PredicateType, TraitBound, TraitBoundModifier, Type, TypeParamBound,
    WhereClause, WherePredicate,
};


/***** LIBRARY FUNCTIONS *****/
/// Collects the generics necessary for the various macro implementations.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
/// - `target`: A [`Path`] encoding the target trait to attach bounds for.
/// - `skip`: An list of fields to skip.
///
/// # Returns
/// A [`Generics`] that can be used for the impl.
pub fn extract_generics(input: &DeriveInput, target: &Path) -> Generics {
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
                    path: target.clone(),
                }));
                bounds
            },
        }));
    }
    generics.where_clause = Some(where_clause);
    generics
}



/// Parses `#[SOME_IDENT(skip)]` on field attributes.
///
/// # Arguments
/// - `base_ident`: The initialization of `SOME_IDENT`.
/// - `attrs`: The list of [`Attribute`]s to skip.
///
/// # Returns
/// True if the skip was found, false otherwise.
///
/// # Errors
/// This function fails if it could not parse the contents of a matching [`Meta::List`] as a list
/// of metas.
pub fn parse_skip_attr(base_ident: &str, attrs: &[Attribute]) -> Result<bool, Error> {
    for attr in attrs {
        match &attr.meta {
            Meta::List(l) if l.path.is_ident(base_ident) => {
                // Parse the contents of the list as a further set of metas
                let attrs: Punctuated<Meta, Comma> = Attribute::parse_args_with(attr, Punctuated::parse_terminated)?;
                for meta in attrs {
                    match meta {
                        Meta::Path(p) if p.is_ident("skip") => return Ok(true),

                        // Anything else we will complain about
                        meta => return Err(Error::new(meta.span(), format!("Unknown attribute {:?}", meta.path()))),
                    }
                }
            },

            // Anything else, we ignore
            _ => continue,
        }
    }
    Ok(false)
}

/// Filters an existing `Data` to skip any variants and fields with a given
/// `#[SOME_IDENT(skip)]`-attribute.
///
/// # Arguments
/// - `base_ident`: The initialization of `SOME_IDENT`.
/// - `data`: Some [`Data`] to filter in.
///
/// # Errors
/// This function fails if it could not parse the contents of a matching [`Meta::List`] as a list
/// of metas.
pub fn filter_skipped_variants_and_fields(base_ident: &str, data: &mut Data) -> Result<(), Error> {
    match data {
        Data::Enum(e) => {
            for variant in &mut e.variants {
                // Also filter the fields of this variant
                match &mut variant.fields {
                    Fields::Named(n) => {
                        let mut fields: Punctuated<Field, Comma> = Punctuated::new();
                        std::mem::swap(&mut fields, &mut n.named);
                        for pair in fields.into_pairs() {
                            if parse_skip_attr(base_ident, &pair.value().attrs)? {
                                continue;
                            }
                            let (value, punct) = pair.into_tuple();
                            n.named.push_value(value);
                            if let Some(punct) = punct {
                                n.named.push_punct(punct);
                            }
                        }
                    },
                    Fields::Unnamed(u) => {
                        let mut fields: Punctuated<Field, Comma> = Punctuated::new();
                        std::mem::swap(&mut fields, &mut u.unnamed);
                        for pair in fields.into_pairs() {
                            if parse_skip_attr(base_ident, &pair.value().attrs)? {
                                continue;
                            }
                            let (value, punct) = pair.into_tuple();
                            u.unnamed.push_value(value);
                            if let Some(punct) = punct {
                                u.unnamed.push_punct(punct);
                            }
                        }
                    },
                    Fields::Unit => {},
                }
            }
            Ok(())
        },

        Data::Struct(s) => match &mut s.fields {
            Fields::Named(n) => {
                let mut fields: Punctuated<Field, Comma> = Punctuated::new();
                std::mem::swap(&mut fields, &mut n.named);
                for pair in fields.into_pairs() {
                    if parse_skip_attr(base_ident, &pair.value().attrs)? {
                        continue;
                    }
                    let (value, punct) = pair.into_tuple();
                    n.named.push_value(value);
                    if let Some(punct) = punct {
                        n.named.push_punct(punct);
                    }
                }
                Ok(())
            },
            Fields::Unnamed(u) => {
                let mut fields: Punctuated<Field, Comma> = Punctuated::new();
                std::mem::swap(&mut fields, &mut u.unnamed);
                for pair in fields.into_pairs() {
                    if parse_skip_attr(base_ident, &pair.value().attrs)? {
                        continue;
                    }
                    let (value, punct) = pair.into_tuple();
                    u.unnamed.push_value(value);
                    if let Some(punct) = punct {
                        u.unnamed.push_punct(punct);
                    }
                }
                Ok(())
            },
            Fields::Unit => Ok(()),
        },

        Data::Union(_) => todo!(),
    }
}
