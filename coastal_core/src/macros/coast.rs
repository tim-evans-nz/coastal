use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Error, Expr, ExprLit, FnArg, Item, ItemConst, ItemFn, Lit, Pat, PatType, Type, TypeImplTrait,
};

use crate::{
    api::{Constant, ConstantValue, Function, State},
    format_err,
};

/// Implementation for `coastal_derive::coast!`.
pub fn coast(attr: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    if !attr.is_empty() {
        return Err(format_err!(
            @attr, "Coastal attribute macro does not take arguments"
        ));
    }
    let item: Item = syn::parse2(input.clone())?;
    match item {
        Item::Const(item_const) => handle_const(item_const)?,
        Item::Fn(item_fn) => handle_fn(item_fn)?,
        _ => {
            return Err(
                format_err!(@item, "#[coast] only supports 'const' and 'fn' items currently"),
            )
        }
    }
    Ok(input)
}

fn handle_fn(item_fn: ItemFn) -> Result<(), Error> {
    if let Some(async_keyword) = item_fn.sig.asyncness {
        return Err(format_err!(
            @async_keyword,
            "Coastal does not support async functions"
        ));
    }
    if item_fn.sig.generics.lt_token.is_some() {
        return Err(format_err!(@item_fn.sig.generics, "Coastal does not support async functions"));
    }
    let mut arguments = Vec::new();
    for arg in item_fn.sig.inputs.iter() {
        match arg {
            FnArg::Receiver(token) => {
                return Err(format_err!(
                    @token, "Coastal does not yet support associated methods"
                ));
            }
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let name = match pat.as_ref() {
                    Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    token => {
                        return Err(format_err!(
                            @token, "Coastal does not support pattern arguments"
                        ))
                    }
                };
                match ty.as_ref() {
                    Type::ImplTrait(TypeImplTrait { impl_token, .. }) => {
                        return Err(format_err!(
                            @impl_token, "Coastal does not support 'impl' arguments"
                        ));
                    }
                    Type::Infer(t) => {
                        return Err(format_err!(
                            @t, "Coastal does not support inferred argument type"
                        ))
                    }
                    Type::Macro(t) => {
                        return Err(format_err!(
                            @t, "Coastal does not support macro argument types"
                        ))
                    }
                    Type::Never(t) => {
                        return Err(
                            format_err!(@t, "Coastal does not support the '!' argument type"),
                        )
                    }
                    Type::TraitObject(t) => {
                        return Err(format_err!(@t, "Coastal does not support 'dyn' arguments"))
                    }
                    _ => (),
                }
                arguments.push((name, ty.to_token_stream().to_string()));
            }
        }
    }
    Function {
        name: item_fn.sig.ident.to_string(),
        return_type: item_fn.sig.output.to_token_stream().to_string(),
        arguments,
    }
    .save_state(&item_fn.sig.ident)?;
    Ok(())
}

fn handle_const(item_const: ItemConst) -> Result<(), Error> {
    let Expr::Lit(ExprLit { lit, .. }) = item_const.expr.as_ref() else {
        return Err(format_err!(
            @item_const.ty,
            "Coastal constants must have a literal value"
        ));
    };
    let value = match (&item_const.ty.to_token_stream().to_string()[..], lit) {
        ("c_char", Lit::Byte(l)) => ConstantValue::CChar(l.value()),
        ("i8", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("i16", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("i32", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("i64", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("u8", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("u16", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("u32", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("u64", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("isize", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("usize", Lit::Int(l)) => ConstantValue::I8(l.base10_parse()?),
        ("f32", Lit::Float(l)) => ConstantValue::F32(l.base10_parse()?),
        ("f64", Lit::Float(l)) => ConstantValue::F32(l.base10_parse()?),
        ("& str", Lit::Str(l)) => ConstantValue::Str(l.value()),
        ("& [u8]", Lit::ByteStr(l)) => ConstantValue::Bytes(l.value()),
        _ => {
            return Err(format_err!(
                @item_const.ty,
                "Coastal constants can only be c_char, int, float, bytes, or string types"
            ))
        }
    };
    Constant {
        name: item_const.ident.to_string(),
        value,
    }
    .save_state(&item_const.ident)?;
    Ok(())
}
