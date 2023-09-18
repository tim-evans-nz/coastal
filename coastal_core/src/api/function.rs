use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::Type;
use syn::{parse_str, Error};

use super::{Library, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: String,
    pub arguments: Vec<(String, String)>,
}

impl Function {
    pub fn rust_wrapper(&self, lib: &Library) -> Result<TokenStream, Error> {
        let name = Ident::new(&self.name, Span::call_site());
        let wrapped_name = Ident::new(
            &format!("{}{}", lib.function_prefix, self.name),
            Span::call_site(),
        );
        let mut declarations = TokenStream::new();
        let mut call = TokenStream::new();
        for (name, ty) in &self.arguments {
            let n = Ident::new(name, Span::call_site());
            let arg_type: Type = parse_str(ty)?;
            let arg = lib.convert_arg(&n, &arg_type)?;
            declarations.extend(arg.decl);
            call.extend(arg.call);
        }
        let ConvertReturn {
            before,
            after,
            return_type,
            ..
        } = lib.convert_return(&parse_str(&self.return_type)?)?;
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
