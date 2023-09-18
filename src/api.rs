use std::io::Write;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use syn::{parse_str, Error};
use syn::{ReturnType, Type};

use crate::errors::format_err;
use crate::state::State;
use crate::types::{convert_builtin_arg, convert_builtin_return};

#[derive(Default)]
pub struct Api {
    pub type_prefix: String,
    pub function_prefix: String,
    pub constant_prefix: String,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
    pub arg_converters: Vec<Box<dyn Fn(&Ident, &Type) -> Option<ConvertArg>>>,
    pub return_converters: Vec<Box<dyn Fn(&ReturnType) -> Option<ConvertReturn>>>,
}

impl Api {
    pub fn new() -> Self {
        let pkg_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "package".to_owned());
        Self {
            type_prefix: pkg_name.to_case(Case::Pascal),
            function_prefix: format!("{}_", pkg_name.to_case(Case::Snake)),
            constant_prefix: format!("{}_", pkg_name.to_case(Case::UpperSnake)),
            arg_converters: vec![Box::new(convert_builtin_arg)],
            return_converters: vec![Box::new(convert_builtin_return)],
            ..Default::default()
        }
    }

    pub fn rust_wrapper(&self) -> Result<TokenStream, Error> {
        let mut output = TokenStream::new();
        for function in &self.functions {
            output.extend(function.rust_wrapper(self)?);
        }
        Ok(quote! {
            mod coastal_wrappers {
                #output
            }
        })
    }

    pub fn convert_arg(&self, name: &Ident, arg_type: &Type) -> Result<ConvertArg, Error> {
        self.arg_converters
            .iter()
            .find_map(|ac| ac(name, arg_type))
            .ok_or_else(|| {
                format_err!(
                    call_site,
                    "can't convert argument '{name}: {}'",
                    arg_type.to_token_stream().to_string()
                )
            })
    }

    pub fn convert_return(&self, return_type: &ReturnType) -> Result<ConvertReturn, Error> {
        self.return_converters
            .iter()
            .find_map(|rc| rc(return_type))
            .ok_or_else(|| {
                format_err!(
                    call_site,
                    "can't convert return type '{}'",
                    return_type.to_token_stream().to_string()
                )
            })
    }
}

pub struct ConvertArg {
    pub decl: TokenStream,
    pub call: TokenStream,
    pub c_args: Vec<String>,
}

pub struct ConvertReturn {
    pub before: TokenStream,
    pub after: TokenStream,
    pub return_type: TokenStream,
    pub c_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constant {
    pub name: String,
    pub value: ConstantValue,
}

impl Constant {
    #[allow(dead_code)]
    pub fn c_header(&self, f: &mut impl Write) -> Result<(), std::io::Error> {
        let name = format!("PREFIX_{}", self.name.to_case(Case::ScreamingSnake));
        match &self.value {
            ConstantValue::CChar(val) => {
                writeln!(f, "#define {name} {:?}", char::from(*val))
            }
            ConstantValue::I8(val) => writeln!(f, "#define {name} ((int8_t){val})"),
            ConstantValue::I16(val) => writeln!(f, "#define {name} ((int16_t){val})"),
            ConstantValue::I32(val) => writeln!(f, "#define {name} {val}"),
            ConstantValue::I64(val) => writeln!(f, "#define {name} {val}ll"),
            ConstantValue::U8(val) => writeln!(f, "#define {name} ((uint8_t){val}u)"),
            ConstantValue::U16(val) => writeln!(f, "#define {name} ((uint16_t){val}u)"),
            ConstantValue::U32(val) => writeln!(f, "#define {name} {val}u"),
            ConstantValue::U64(val) => writeln!(f, "#define {name} {val}ull"),
            ConstantValue::F32(val) => writeln!(f, "#define {name} {val}f"),
            ConstantValue::F64(val) => writeln!(f, "#define {name} {val}"),
            ConstantValue::Str(val) => writeln!(f, "#define {name} {val:?}"),
            ConstantValue::Bytes(val) => {
                let string_constant =
                    String::from_utf8(val.iter().copied().flat_map(u8::escape_ascii).collect())
                        .expect("valid utf8");
                writeln!(f, "#define {name} \"{string_constant}\"")
            }
        }
    }
}

impl State for Constant {
    const TYPE_NAME: &'static str = "coastal.constant";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstantValue {
    CChar(u8),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(i8),
    U16(i16),
    U32(i32),
    U64(i64),
    F32(f32),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: String,
    pub arguments: Vec<(String, String)>,
}

impl Function {
    pub fn rust_wrapper(&self, api: &Api) -> Result<TokenStream, Error> {
        let name = Ident::new(&self.name, Span::call_site());
        let wrapped_name = Ident::new(
            &format!("{}{}", api.function_prefix, self.name),
            Span::call_site(),
        );
        let mut declarations = TokenStream::new();
        let mut call = TokenStream::new();
        for (name, ty) in &self.arguments {
            let n = Ident::new(&name, Span::call_site());
            let arg_type: Type = parse_str(&ty)?;
            let arg = api.convert_arg(&n, &arg_type)?;
            declarations.extend(arg.decl);
            call.extend(arg.call);
        }
        let ConvertReturn {
            before,
            after,
            return_type,
            ..
        } = api.convert_return(&parse_str(&self.return_type)?)?;
        Ok(quote! {
            #[no_mangle]
            pub extern "C" fn #wrapped_name(#declarations) -> #return_type {
                #before super::#name(#call) #after
            }
        })
    }
}

impl State for Function {
    const TYPE_NAME: &'static str = "coastal.function";
}
