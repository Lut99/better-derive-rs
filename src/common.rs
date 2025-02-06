//  COMMON.rs
//    by Lut99
//
//  Created:
//    09 Jan 2025, 01:10:02
//  Last edited:
//    06 Feb 2025, 15:33:18
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines common functionality between the macros.
//

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::visit::Visit;
use syn::{
    Attribute, Data, DeriveInput, Error, Field, Fields, GenericParam, Generics, Ident, Meta, Path, PredicateType, Token, TraitBound,
    TraitBoundModifier, Type, TypeParamBound, WherePredicate,
};


/***** CONSTANTS *****/
/// The name of the generic, cross-trait attribute.
pub const COMMON_ATTR_NAME: &'static str = "better_derive";





/***** HELPER FUNCTIONS *****/
/// Parses the data-level attributes in search of answers.
///
/// # Arguments
/// - `base_ident`: The name of any attributes to look for (e.g., `debug`). Note that the
///   [common attribute](COMMON_ATTR_NAME) is always included.
/// - `attrs`: The list of [`Attribute`]s to parse.
///
/// # Returns
/// A list of [`Type`]s that represent the types to introduce the usual suspect trait bounds for,
/// or [`None`] if none were given (and automatic derivation should be used).
///
/// # Errors
/// Note that this function can error if an attribute belonging to th(i|e)s(e) macro(s) was given,
/// but we failed to understand it.
fn parse_toplevel_attrs(base_ident: &str, attrs: &[Attribute]) -> Result<Option<Vec<Type>>, Error> {
    let mut found: bool = false;
    let mut results: Vec<Type> = Vec::new();
    for attr in attrs {
        match &attr.meta {
            Meta::List(l) if l.path.is_ident(COMMON_ATTR_NAME) || l.path.is_ident(base_ident) => {
                // Parse the contents of the list as a further set of metas
                let attrs: Punctuated<ToplevelAttr, Token![,]> = Attribute::parse_args_with(attr, Punctuated::parse_terminated)?;
                for attr in attrs {
                    match attr {
                        ToplevelAttr::Bound(tys) => {
                            found = true;
                            results.extend(tys);
                        },
                    }
                }
            },

            // Anything else, we ignore
            _ => continue,
        }
    }

    // Return appropriately
    if found { Ok(Some(results)) } else { Ok(None) }
}

/// Parses `#[SOME_IDENT(skip)]` on field attributes.
///
/// # Arguments
/// - `base_ident`: The initialization of `SOME_IDENT`. Note that the
///   [common attribute](COMMON_ATTR_NAME) is always included.
/// - `attrs`: The list of [`Attribute`]s to skip.
///
/// # Returns
/// True if the skip was found, false otherwise.
///
/// # Errors
/// This function fails if it could not parse the contents of a matching [`Meta::List`] as a list
/// of metas.
fn parse_field_attrs(base_ident: &str, attrs: &[Attribute]) -> Result<bool, Error> {
    for attr in attrs {
        match &attr.meta {
            Meta::List(l) if l.path.is_ident(COMMON_ATTR_NAME) || l.path.is_ident(base_ident) => {
                // Parse the contents of the list as a further set of metas
                let attrs: Punctuated<Meta, Token![,]> = Attribute::parse_args_with(attr, Punctuated::parse_terminated)?;
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




/***** HELPERS *****/
/// Defines a parsable attribute for the toplevel.
enum ToplevelAttr {
    /// The user is defining a type to bind.
    Bound(Punctuated<Type, Token![,]>),
}
impl Parse for ToplevelAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "bound" || ident == "bounds" {
            input.parse::<Token![=]>()?;
            let content;
            syn::parenthesized!(content in input);
            Ok(Self::Bound(Punctuated::parse_terminated(&content)?))
        } else {
            Err(input.error(format!("Unknown attribute {ident:?}")))
        }
    }
}



/// Defines a visitor for finding if a type uses any generics.
struct HasGenericsVisitor<'g> {
    /// The generics to check for membership in.
    generics: &'g Generics,
    /// Whether we found any generics or not.
    found:    bool,
}
impl<'g> HasGenericsVisitor<'g> {
    /// Does the search, returns the answer.
    ///
    /// # Arguments
    /// - `ty`: Some [`Type`] to check if it uses any `generics`.
    /// - `generics`: Some [`Generics`] describing the parameters the given `ty`pe may use.
    ///
    /// # Returns
    /// True if `ty` has any parameters from `generics`, or false otherwise.
    #[inline]
    fn has_generics(ty: &Type, generics: &'g Generics) -> bool {
        let mut visitor = Self { generics, found: false };
        visitor.visit_type(ty);
        visitor.found
    }
}
impl<'ast, 'g> Visit<'ast> for HasGenericsVisitor<'g> {
    #[inline]
    fn visit_type(&mut self, ty: &'ast Type) {
        // Check if the type is an identifier
        if let Type::Path(ty_path) = ty {
            if let Some(ty_ident) = ty_path.path.get_ident() {
                // If so, check if it occurs anywhere in the generics
                if self.generics.params.iter().any(|p| if let GenericParam::Type(gen_ty) = p { ty_ident == &gen_ty.ident } else { false }) {
                    // No need to continue, we found it
                    self.found = true;
                    return;
                }
            }
        }

        // If we didn't find it, then recurse as usual
        syn::visit::visit_type(self, ty)
    }
}





/***** LIBRARY FUNCTIONS *****/
/// Collects the generics necessary for the various macro implementations.
///
/// # Arguments
/// - `base_ident`: The name of any attributes to look for (e.g., `debug`). Note that the
///   [common attribute](COMMON_ATTR_NAME) is always included.
/// - `attrs`: A list of toplevel attributes to optionally extract generics from.
/// - `input`: The [`DeriveInput`] which we will scan to collect the genreics.
/// - `target`: A [`Path`] encoding the target trait to attach bounds for.
///
/// # Returns
/// A [`Generics`] that can be used for the impl.
pub fn extract_generics(base_ident: &str, attrs: &[Attribute], input: &DeriveInput, target: &Path) -> Result<Generics, Error> {
    let mut generics = input.generics.clone();

    // Find the list from the attributes if given
    if let Some(tys) = parse_toplevel_attrs(base_ident, attrs)? {
        // Extend the where-clauses with the appropriate bounds
        generics.make_where_clause().predicates.extend(tys.into_iter().map(|ty| {
            WherePredicate::Type(PredicateType {
                lifetimes:   None,
                bounded_ty:  ty,
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
            })
        }));

        // OK, done!
        return Ok(generics);
    }

    // Simply find a list of all types in the struct
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
        // Skip this type if it doesn't contain any generics (then it's not up to us to define
        // additional bounds)
        if !HasGenericsVisitor::has_generics(ty, &generics) {
            continue;
        }

        // It does, so add it as a bound
        generics.make_where_clause().predicates.push(WherePredicate::Type(PredicateType {
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
    Ok(generics)
}



/// Filters an existing `Data` to skip any variants and fields with a given
/// `#[SOME_IDENT(skip)]`-attribute.
///
/// # Arguments
/// - `base_ident`: The initialization of `SOME_IDENT`. Note that the
///   [common attribute](COMMON_ATTR_NAME) is always included.
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
                        let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
                        std::mem::swap(&mut fields, &mut n.named);
                        for pair in fields.into_pairs() {
                            if parse_field_attrs(base_ident, &pair.value().attrs)? {
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
                        let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
                        std::mem::swap(&mut fields, &mut u.unnamed);
                        for pair in fields.into_pairs() {
                            if parse_field_attrs(base_ident, &pair.value().attrs)? {
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
                let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
                std::mem::swap(&mut fields, &mut n.named);
                for pair in fields.into_pairs() {
                    if parse_field_attrs(base_ident, &pair.value().attrs)? {
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
                let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
                std::mem::swap(&mut fields, &mut u.unnamed);
                for pair in fields.into_pairs() {
                    if parse_field_attrs(base_ident, &pair.value().attrs)? {
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
