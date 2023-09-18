use proc_macro2::TokenStream;
use syn::{parse::Parse, parse2, token, Error, Ident, Lit};

use crate::{
    api::{Api, Constant, Function},
    errors::err,
    state::State,
};

pub fn macro_coastal_impl(input: TokenStream) -> Result<TokenStream, Error> {
    let api: Api = parse2(input)?;
    api.rust_wrapper()
}

impl Parse for Api {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut api = Api::new();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(token::Fn) {
                input.parse::<token::Fn>()?;
                let ident: Ident = input.parse()?;
                input.parse::<token::Semi>()?;
                api.functions.push(Function::load_state(&ident)?)
            } else if lookahead.peek(token::Const) {
                input.parse::<token::Const>()?;
                let ident: Ident = input.parse()?;
                input.parse::<token::Semi>()?;
                api.constants.push(Constant::load_state(&ident)?)
            } else if lookahead.peek(token::Let) {
                input.parse::<token::Let>()?;
                let ident: Ident = input.parse()?;
                input.parse::<token::Eq>()?;
                let _value_lit: Lit = input.parse()?;
                input.parse::<token::Semi>()?;
                match &ident.to_string()[..] {
                    // "prefix" => api
                    //     .set_prefix(&value_lit.value())
                    //     .map_err(|s| Error::new_spanned(value_lit, s))?,
                    other => err!(ident, "Coastal did not recognise option '{other}'"),
                }
            }
        }
        Ok(api)
    }
}
