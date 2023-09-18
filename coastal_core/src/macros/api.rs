use proc_macro2::TokenStream;
use syn::{parse::Parse, parse2, token, Error, Ident};

use crate::api::{Constant, Function, Library, State};

/// Implementation for `coastal_derive::coast!`.
pub fn api(input: TokenStream) -> Result<TokenStream, Error> {
    let api: Api = parse2(input)?;
    api.library.rust_wrapper()
}

#[derive(Default)]
struct Api {
    library: Library,
}

impl Parse for Api {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut api = Api::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(token::Fn) {
                input.parse::<token::Fn>()?;
                let ident: Ident = input.parse()?;
                input.parse::<token::Semi>()?;
                api.library.functions.push(Function::load_state(&ident)?)
            } else if lookahead.peek(token::Const) {
                input.parse::<token::Const>()?;
                let ident: Ident = input.parse()?;
                input.parse::<token::Semi>()?;
                api.library.constants.push(Constant::load_state(&ident)?)
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(api)
    }
}
