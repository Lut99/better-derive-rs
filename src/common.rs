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

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::visit::Visit;
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Error, Expr, ExprPath, Field, Fields, GenericArgument, GenericParam, Generics,
    Ident, Meta, Path, PathArguments, PathSegment, PredicateType, Token, TraitBound, TraitBoundModifier, Type, TypeParamBound, TypePath,
    WherePredicate, parenthesized,
};


/***** CONSTANTS *****/
/// The name of the generic, cross-trait attribute.
pub const COMMON_ATTR_NAME: &'static str = "better_derive";





/***** HELPER FUNCTIONS *****/
/// Converts a generic parameter into a generic argument.
///
/// # Arguments
/// - `param`: A (reference to a) [`GenericParam`] to convert.
///
/// # Returns
/// An equivlanet [`GenericArgument`]. Note that it has fundamentally less information.
#[inline]
fn gen_param_to_arg(param: &GenericParam) -> GenericArgument {
    match param {
        GenericParam::Const(c) => GenericArgument::Const(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path:  Path {
                leading_colon: None,
                segments:      {
                    let mut segments = Punctuated::new();
                    segments.push(PathSegment { ident: c.ident.clone(), arguments: PathArguments::None });
                    segments
                },
            },
        })),
        GenericParam::Lifetime(l) => GenericArgument::Lifetime(l.lifetime.clone()),
        GenericParam::Type(t) => GenericArgument::Type(Type::Path(TypePath {
            qself: None,
            path:  Path {
                leading_colon: None,
                segments:      {
                    let mut segments = Punctuated::new();
                    segments.push(PathSegment { ident: t.ident.clone(), arguments: PathArguments::None });
                    segments
                },
            },
        })),
    }
}



/// Parses the data-level attributes in search of answers.
///
/// # Arguments
/// - `base_ident`: The name of any attributes to look for (e.g., `debug`). Note that the
///   [common attribute](COMMON_ATTR_NAME) is always included.
/// - `attrs`: The list of [`Attribute`]s to parse.
///
/// # Returns
/// A list of params and a matching where clause that shalt be the bound for this impl, or [`None`]
/// if none were given (and automatic derivation should be used).
///
/// # Errors
/// Note that this function can error if an attribute belonging to th(i|e)s(e) macro(s) was given,
/// but we failed to understand it.
fn parse_toplevel_attrs(base_ident: &str, attrs: &[Attribute]) -> Result<ToplevelAttrs, Error> {
    let mut impl_gen: Option<Punctuated<GenericParam, Token![,]>> = None;
    let mut ty_gen: Option<Punctuated<GenericArgument, Token![,]>> = None;
    let mut where_clause: Option<Punctuated<WherePredicate, Token![,]>> = None;
    for attr in attrs {
        match &attr.meta {
            Meta::List(l) if l.path.is_ident(COMMON_ATTR_NAME) || l.path.is_ident(base_ident) => {
                // Parse the contents of the list as a further set of metas
                let attrs: Punctuated<ToplevelAttr, Token![,]> = Attribute::parse_args_with(attr, Punctuated::parse_terminated)?;
                for attr in attrs {
                    match attr {
                        ToplevelAttr::ImplGen(params) => {
                            impl_gen = Some(params);
                        },
                        ToplevelAttr::TypeGen(args) => {
                            ty_gen = Some(args);
                        },
                        ToplevelAttr::WhereClause(preds) => {
                            where_clause = Some(preds);
                        },
                    }
                }
            },

            // Anything else, we ignore
            _ => continue,
        }
    }

    // Return appropriately
    Ok(ToplevelAttrs { impl_gen, ty_gen, where_clause })
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
/// Defines a collection of all information we parse toplevel.
struct ToplevelAttrs {
    impl_gen: Option<Punctuated<GenericParam, Token![,]>>,
    ty_gen: Option<Punctuated<GenericArgument, Token![,]>>,
    where_clause: Option<Punctuated<WherePredicate, Token![,]>>,
}

/// Defines a parsable attribute for the toplevel.
enum ToplevelAttr {
    /// The impl type generics.
    ImplGen(Punctuated<GenericParam, Token![,]>),
    /// The type-attached generics.
    TypeGen(Punctuated<GenericArgument, Token![,]>),
    /// The user is defining type constraints.
    WhereClause(Punctuated<WherePredicate, Token![,]>),
}
impl Parse for ToplevelAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "impl_gen" {
            // Parse the equals sign and then the type generics clauses
            input.parse::<Token![=]>()?;
            // NOTE: Apparently doesn't do where!
            let generics = input.parse::<Generics>()?;
            Ok(Self::ImplGen(generics.params))
        } else if ident == "type_gen" {
            // Parse the equals sign and then the type generics clauses
            input.parse::<Token![=]>()?;
            let args = input.parse::<AngleBracketedGenericArguments>()?;
            Ok(Self::TypeGen(args.args))
        } else if ident == "bound" || ident == "bounds" {
            // Parse the equals sign, parenthesis and then the where clauses
            input.parse::<Token![=]>()?;
            let content;
            parenthesized!(content in input);
            Ok(Self::WhereClause(Punctuated::parse_terminated(&content)?))
        } else {
            Err(input.error(format!("Unknown attribute {ident:?}")))
        }
    }
}



/// Defines a visitor for finding if a type uses any generics.
struct HasGenericsVisitor<'g> {
    /// The generics to check for membership in.
    generics: &'g Punctuated<GenericParam, Token![,]>,
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
    fn has_generics(ty: &Type, generics: &'g Punctuated<GenericParam, Token![,]>) -> bool {
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
                if self.generics.iter().any(|p| if let GenericParam::Type(gen_ty) = p { ty_ident == &gen_ty.ident } else { false }) {
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





/***** AUXILLARY *****/
/// Helper type for correctly serializing a list of generic parameters.
pub struct ImplGen(Punctuated<GenericParam, Token![,]>);
impl ToTokens for ImplGen {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if !self.0.is_empty() {
            let impl_gen = &self.0;
            tokens.extend(quote! { <#impl_gen> })
        }
    }
}

/// Helper type for correctly serializing a list of generic arguments.
pub struct TypeGen(Punctuated<GenericArgument, Token![,]>);
impl ToTokens for TypeGen {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if !self.0.is_empty() {
            let type_gen = &self.0;
            tokens.extend(quote! { <#type_gen> })
        }
    }
}

/// Helper type for correctly serializing a list of where predicates.
pub struct WhereClause(Punctuated<WherePredicate, Token![,]>);
impl ToTokens for WhereClause {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if !self.0.is_empty() {
            let preds = &self.0;
            tokens.extend(quote! { where #preds })
        }
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
pub fn extract_generics(base_ident: &str, attrs: &[Attribute], input: &DeriveInput, target: &Path) -> Result<(ImplGen, TypeGen, WhereClause), Error> {
    // parse the arguments
    let ToplevelAttrs { impl_gen, ty_gen, where_clause } = parse_toplevel_attrs(base_ident, attrs)?;

    // Then either use the given parameters, replacing `r#trait` where needed; or copy the impl
    // ones
    let impl_gen: Punctuated<GenericParam, Token![,]> = if let Some(mut impl_gen) = impl_gen {
        // Replace any occurrance of `r#trait` with the current one.
        for param in &mut impl_gen {
            if let GenericParam::Type(param) = param {
                for bound in &mut param.bounds {
                    if let TypeParamBound::Trait(trt) = bound {
                        if trt.path.is_ident("r#trait") {
                            trt.path = target.clone();
                        }
                    }
                }
            }
        }
        impl_gen
    } else {
        // By default, we copy whatever is in the impl
        input.generics.params.clone()
    };

    // For the types, we copy the given or use the params as arguments, filtering those not part of
    // the typedef
    let ty_gen: Punctuated<GenericArgument, Token![,]> = if let Some(ty_gen) = ty_gen {
        ty_gen
    } else {
        // By default, we copy whatever is in the impl ~ but only those actually present in the
        // type's definition
        impl_gen
            .iter()
            .filter(|param| {
                for orig_param in &input.generics.params {
                    if orig_param == *param {
                        return true;
                    }
                }
                false
            })
            .map(gen_param_to_arg)
            .collect()
    };

    // Finally, the where clause is straightforward to copy (also replace `r#trait`), but
    // generating it is where the magic happens
    let where_clause: Punctuated<WherePredicate, Token![,]> = if let Some(mut where_clause) = where_clause {
        // Replace any occurrance of `r#trait` with the current one.
        for param in &mut where_clause {
            if let WherePredicate::Type(param) = param {
                for bound in &mut param.bounds {
                    if let TypeParamBound::Trait(trt) = bound {
                        if trt.path.is_ident("r#trait") {
                            trt.path = target.clone();
                        }
                    }
                }
            }
        }
        where_clause
    } else {
        // Simply find a list of all types in the struct
        let mut preds = Punctuated::new();
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
            if !HasGenericsVisitor::has_generics(ty, &impl_gen) {
                continue;
            }

            // It does, so add it as a bound
            preds.push(WherePredicate::Type(PredicateType {
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
        preds
    };

    // Done! Return that
    Ok((ImplGen(impl_gen), TypeGen(ty_gen), WhereClause(where_clause)))
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
