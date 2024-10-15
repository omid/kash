pub mod no_cache_fn;

use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::ops::Deref;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_str, Block, Expr, FnArg, Pat, PatType, Type};

pub(super) fn gen_cache_ident(name: &Option<String>, fn_ident: &Ident) -> Ident {
    let name = name.clone().unwrap_or(fn_ident.to_string()).to_uppercase();
    Ident::new(&name, fn_ident.span())
}

pub(super) fn match_pattern_type(pat_type: &PatType) -> Box<Pat> {
    match &pat_type.pat.deref() {
        Pat::Ident(pat_ident) => {
            if pat_ident.mutability.is_some() {
                let mut p = pat_ident.clone();
                p.mutability = None;
                Box::new(Pat::Ident(p))
            } else {
                Box::new(Pat::Ident(pat_ident.clone()))
            }
        }
        _ => pat_type.pat.clone(),
    }
}

// make the block that converts the inputs into the key type
pub(super) fn make_cache_key_type(
    convert: &Option<String>,
    key: &Option<String>,
    input_tys: Vec<Type>,
    input_names: &Vec<TokenStream2>,
) -> (TokenStream2, TokenStream2) {
    match (key, convert) {
        (Some(key_str), Some(convert_str)) => {
            let cache_key_ty = dereference_type(
                parse_str::<Type>(key_str).expect("unable to parse a cache key type"),
            );

            let key_convert_block =
                parse_str::<Expr>(convert_str).expect("Unable to parse key convert block");

            (quote! {#cache_key_ty}, quote! {#key_convert_block})
        }
        (None, Some(convert_str)) => {
            let key_convert_block =
                parse_str::<Block>(convert_str).expect("Unable to parse key convert block");

            (quote! {}, quote! {#key_convert_block})
        }
        (None, None) => {
            let input_tys = input_tys.into_iter().map(dereference_type);
            (
                quote! {(#(#input_tys),*)},
                quote! {(#(#input_names.clone()),*)},
            )
        }
        (_, _) => panic!("key requires convert to be set"),
    }
}

/// Convert a type `&T` into a type `T`.
///
/// If the input is a tuple, the elements are de-referenced.
///
/// Otherwise, the input is returned unchanged.
pub(super) fn dereference_type(ty: Type) -> Type {
    match ty {
        Type::Reference(r) => *r.elem,
        Type::Tuple(mut tt) => {
            tt.elems = tt
                .elems
                .iter()
                .map(|ty| dereference_type(ty.clone()))
                .collect();
            Type::Tuple(tt)
        }
        _ => ty,
    }
}

// if you define arguments as mutable, e.g.
// #[kash]
// fn mutable_args(mut a: i32, mut b: i32) -> (i32, i32) {
//     a += 1;
//     b += 1;
//     (a, b)
// }
// then we need to strip off the `mut` keyword from the
// variable identifiers, so we can refer to arguments `a` and `b`
// instead of `mut a` and `mut b`
pub(super) fn get_input_names(
    inputs: &Punctuated<FnArg, Comma>,
) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    let maybe_with_self_names = inputs
        .iter()
        .map(|input| match input {
            FnArg::Receiver(r) => r.self_token.to_token_stream(),
            FnArg::Typed(pat_type) => match_pattern_type(pat_type).to_token_stream(),
        })
        .collect();
    let without_self_names = inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(match_pattern_type(pat_type).to_token_stream()),
        })
        .collect();
    (maybe_with_self_names, without_self_names)
}

// pull out the names and types of the function inputs
pub(super) fn get_input_types(inputs: &Punctuated<FnArg, Comma>) -> (Vec<Type>, Vec<Type>) {
    let maybe_with_self_types = inputs
        .iter()
        .map(|input| match input {
            FnArg::Receiver(r) => *r.ty.clone(),
            FnArg::Typed(pat_type) => *pat_type.ty.clone(),
        })
        .collect();
    let without_self_types = inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(*pat_type.ty.clone()),
        })
        .collect();
    (maybe_with_self_types, without_self_types)
}

pub(super) fn get_output_parts(output_ts: &TokenStream) -> Vec<String> {
    output_ts
        .clone()
        .into_iter()
        .filter_map(|tt| match tt {
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None,
        })
        .collect()
}
