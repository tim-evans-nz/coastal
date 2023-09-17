use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    Abi, BareFnArg, Expr, Lifetime, Path, ReturnType, Type, TypeArray, TypeBareFn, TypeGroup,
    TypeParen, TypePath, TypePtr, TypeReference,
};

use crate::api::{ConvertArg, ConvertReturn};

/// Returns the code to convert an argument from C to Rust.
pub fn convert_builtin_arg(name: &Ident, arg_type: &Type) -> Option<ConvertArg> {
    match arg_type {
        Type::Array(TypeArray { elem, len, .. }) => convert_array_arg(name, elem, len),
        Type::BareFn(TypeBareFn {
            lifetimes: Some(_), ..
        }) => None, // function with lifetime
        Type::BareFn(TypeBareFn {
            variadic: Some(_), ..
        }) => None, // variadic function
        Type::BareFn(TypeBareFn {
            unsafety,
            abi,
            inputs,
            output,
            ..
        }) => convert_fn_ptr(name, inputs, output, abi.as_ref(), unsafety.is_some()),
        Type::Group(TypeGroup { elem, .. }) | Type::Paren(TypeParen { elem, .. }) => {
            convert_builtin_arg(name, &elem)
        }
        Type::Path(TypePath { qself: Some(_), .. }) => None, // <T as Trait>::U
        Type::Path(TypePath { path, .. }) => convert_path_arg(name, path),
        Type::Ptr(TypePtr {
            mutability, elem, ..
        }) => convert_ptr_arg(name, &elem, mutability.is_some()),
        Type::Reference(TypeReference {
            lifetime,
            mutability,
            elem,
            ..
        }) => convert_ref_arg(name, lifetime.as_ref(), &elem, mutability.is_some()),
        Type::Slice(_) => None,       // bare slice [T]
        Type::ImplTrait(_) => None,   // impl Trait
        Type::TraitObject(_) => None, // dyn Trait
        Type::Tuple(_) => None,       // (T, U, ...)
        _ => None,
    }
}

pub fn convert_builtin_return(return_type: &ReturnType) -> Option<ConvertReturn> {
    match return_type {
        ReturnType::Default => Some(ConvertReturn {
            before: quote! {},
            after: quote! { ; },
            return_type: quote! { () },
            c_type: "void".to_owned(),
        }),
        ReturnType::Type(_, ty) => convert_builtin_result_type(ty.as_ref()),
    }
}

fn convert_builtin_result_type(ty: &Type) -> Option<ConvertReturn> {
    match ty {
        Type::Array(_) => None,  // can't return [T; N]
        Type::BareFn(_) => None, // can't return fn(...) -> T
        Type::Group(TypeGroup { elem, .. }) | Type::Paren(TypeParen { elem, .. }) => {
            convert_builtin_result_type(elem)
        }
        Type::ImplTrait(_) => None, // impl T
        Type::Never(_) => Some(ConvertReturn {
            before: quote! {},
            after: quote! { ; },
            return_type: quote! { ! },
            c_type: "void".to_owned(),
        }), // no return
        Type::Path(TypePath { qself: Some(_), .. }) => None,
        Type::Path(TypePath { path, .. }) => convert_path_return(path), // return by value
        Type::Ptr(_) => todo!("return pointer"),
        Type::Reference(_) => todo!("return ref"), // return pointer
        Type::Slice(_) => None,                    // [T]
        Type::TraitObject(_) => None,              // dyn Trait
        Type::Tuple(_) => None,                    // (T, U, ...)
        _ => None,
    }
}

fn convert_array_arg(_name: &Ident, _elem: &Type, _len: &Expr) -> Option<ConvertArg> {
    todo!("convert_array_arg")
}

fn convert_fn_ptr<'a>(
    _name: &Ident,
    _inputs: impl IntoIterator<Item = &'a BareFnArg>,
    _output: &ReturnType,
    _abi: Option<&Abi>,
    _is_unsafe: bool,
) -> Option<ConvertArg> {
    todo!("convert_fn_ptr")
}

/// Converts arguments like `foo: i32` by value.
fn convert_path_arg(name: &Ident, type_path: &Path) -> Option<ConvertArg> {
    const NO_CONVERSION: &[(&str, &str, &str)] = &[
        ("bool", "bool", "bool"),
        ("i8", "i8", "int8_t"),
        ("i16", "i16", "int16_t"),
        ("i32", "i32", "int32_t"),
        ("i64", "i64", "int64_t"),
        ("u8", "u8", "uint8_t"),
        ("u16", "u16", "uint16_t"),
        ("u32", "u32", "uint32_t"),
        ("u64", "u64", "uint64_t"),
        ("usize", "usize", "size_t"),
        ("isize", "isize", "ssize_t"),
        ("f32", "f32", "float"),
        ("f64", "f64", "double"),
        ("c_char", "std::ffi::c_char", "char"),
        ("c_uchar", "std::ffi::c_uchar", "unsigned char"),
        ("c_schar", "std::ffi::c_schar", "signed char"),
    ];
    const NON_ZERO: &[(&str, &str, &str)] = &[
        ("NonZeroI8", "i8", "int8_t"),
        ("NonZeroI16", "i16", "int16_t"),
        ("NonZeroI32", "i32", "int32_t"),
        ("NonZeroI64", "i64", "int64_t"),
        ("NonZeroU8", "u8", "int8_t"),
        ("NonZeroU16", "u16", "int16_t"),
        ("NonZeroU32", "u32", "int32_t"),
        ("NonZeroU64", "u64", "int64_t"),
        ("NonZeroIsize", "isize", "ssize_t"),
        ("NonZeroUsize", "usize", "size_t"),
    ];
    let type_string = type_path.into_token_stream().to_string();
    if let Some(r) = NO_CONVERSION
        .iter()
        .find(|(n, _, _)| n == &&type_string)
        .map(|(_, p, c)| {
            let path = TokenStream::from_str(p).unwrap();
            ConvertArg {
                decl: quote! { #name: #path, },
                call: quote! { #name, },
                c_args: vec![format!("{c} {name}")],
            }
        })
    {
        return Some(r);
    }
    if let Some(r) = NON_ZERO
        .iter()
        .find(|(n, _, _)| n == &&type_string)
        .map(|(_, r, c)| {
            let raw_type = Ident::new(r, Span::call_site());
            ConvertArg {
                decl: quote! {
                    #name: #raw_type,
                },
                call: quote! {
                    #name.try_into().map_err(|_| panic!("argument '#name' must be non-zero")),
                },
                c_args: vec![format!("{c} {name}")],
            }
        })
    {
        return Some(r);
    }
    None
}

fn convert_ptr_arg(_name: &Ident, _path: &Type, _mutable: bool) -> Option<ConvertArg> {
    todo!("convert_ptr_arg")
}

fn convert_ref_arg(
    _name: &Ident,
    _lifetime: Option<&Lifetime>,
    _elem: &Type,
    _mutable: bool,
) -> Option<ConvertArg> {
    todo!("convert_ref_arg")
}

fn convert_path_return(type_path: &Path) -> Option<ConvertReturn> {
    const ACCEPT: &[(&str, &str, &str)] = &[
        ("bool", "bool", "bool"),
        ("i8", "i8", "int8_t"),
        ("i16", "i16", "int16_t"),
        ("i32", "i32", "int32_t"),
        ("i64", "i64", "int64_t"),
        ("u8", "u8", "uint8_t"),
        ("u16", "u16", "uint12_t"),
        ("u32", "u32", "uint32_t"),
        ("u64", "u64", "uint64_t"),
        ("usize", "usize", "size_t"),
        ("isize", "isize", "ssize_t"),
        ("f32", "f32", "float"),
        ("f64", "f64", "double"),
        ("c_char", "c_char", "char"),
        ("c_uchar", "c_uchar", "unsigned char"),
        ("c_schar", "c_schar", "signed char"),
        ("NonZeroI8", "i8", "int8_t"),
        ("NonZeroI16", "i16", "int16_t"),
        ("NonZeroI32", "i32", "int32_t"),
        ("NonZeroI64", "i64", "int64_t"),
        ("NonZeroU8", "u8", "unt8_t"),
        ("NonZeroU16", "u16", "unt16_t"),
        ("NonZeroU32", "u32", "unt32_t"),
        ("NonZeroU64", "u64", "unt64_t"),
        ("NonZeroUsize", "usize", "size_t"),
        ("NonZeroIsize", "isize", "ssize_t"),
    ];
    let type_string = type_path.into_token_stream().to_string();
    ACCEPT
        .iter()
        .find(|(n, _, _)| n == &&type_string)
        .map(|(n, r, c)| ConvertReturn {
            before: quote! {},
            after: if n == r {
                quote! {}
            } else {
                quote! { .into() }
            },
            return_type: TokenStream::from_str(r).unwrap(),
            c_type: (*c).to_owned(),
        })
}
