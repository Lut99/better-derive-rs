//  EXTRACT.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:10:02
//  Last edited:
//    09 Jan 2025, 01:20:59
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines functionality to read the code to get the fields and such.
//

use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Fields, Generics, Path, PredicateType, TraitBound, TraitBoundModifier, Type, TypeParamBound, WhereClause, WherePredicate,
};


/***** LIBRARY *****/
/// Collects the generics necessary for the [`Debug`](std::fmt::Debug) implementation.
///
/// # Arguments
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
/// - `target`: A [`Path`] encoding the target trait to attach bounds for.
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
