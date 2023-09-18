use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Error;
use syn::{ReturnType, Type};

use crate::{
    format_err,
    types::{convert_builtin_arg, convert_builtin_return},
};

use super::{Constant, ConvertArg, ConvertReturn, Function};

pub type ArgConverter = Box<dyn Fn(&Ident, &Type) -> Option<ConvertArg>>;
pub type ReturnConverter = Box<dyn Fn(&ReturnType) -> Option<ConvertReturn>>;

pub struct Library {
    pub type_prefix: String,
    pub function_prefix: String,
    pub constant_prefix: String,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
    pub arg_converters: Vec<ArgConverter>,
    pub return_converters: Vec<ReturnConverter>,
}

impl Library {
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
                    "can't convert return type '{}'",
                    return_type.to_token_stream().to_string()
                )
            })
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}
