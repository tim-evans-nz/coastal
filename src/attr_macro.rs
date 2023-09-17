use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Error, FnArg, ItemFn, Pat, PatType, Type, TypeImplTrait};

use crate::{api::Function, errors::err, state::State};

pub fn attr_macro_coastal(attr: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    if !attr.is_empty() {
        err!(attr, "Coastal attribute macro does not take arguments");
    }
    let input_func = syn::parse2::<ItemFn>(input.clone())?;
    if let Some(async_keyword) = input_func.sig.asyncness {
        err!(async_keyword, "Coastal does not support async functions");
    }
    if input_func.sig.generics.lt_token.is_some() {
        err!(
            input_func.sig.generics,
            "Coastal does not support async functions"
        );
    }
    let mut arguments = Vec::new();
    for arg in input_func.sig.inputs.iter() {
        match arg {
            FnArg::Receiver(token) => {
                err!(token, "Coastal does not yet support associated methods")
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let name = match pat.as_ref() {
                    Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    token => err!(token, "Coastal does not support pattern arguments"),
                };
                match ty.as_ref() {
                    Type::ImplTrait(TypeImplTrait { impl_token, .. }) => {
                        err!(impl_token, "Coastal does not support 'impl' arguments")
                    }
                    Type::Infer(t) => err!(t, "Coastal does not support inferred argument type"),
                    Type::Macro(t) => err!(t, "Coastal does not support macro argument types"),
                    Type::Never(t) => err!(t, "Coastal does not support the '!' argument type"),
                    Type::TraitObject(t) => err!(t, "Coastal does not support 'dyn' arguments"),
                    _ => (),
                }
                arguments.push((name, ty.to_token_stream().to_string()));
            }
        }
    }
    Function {
        name: input_func.sig.ident.to_string(),
        return_type: input_func.sig.output.to_token_stream().to_string(),
        arguments,
    }
    .save_state(&input_func.sig.ident)?;
    Ok(input)
}
