use proc_macro2::{TokenStream, TokenTree};
use syn::Error;

use crate::{
    api::{Api, Function},
    errors::err,
    state::State,
};

pub fn macro_coastal_impl(input: TokenStream) -> Result<TokenStream, Error> {
    let mut api = Api::new();
    for token in input {
        let TokenTree::Ident(ident) = &token else {
            err!(token, "expected an indentifier");
        };
        api.functions.push(Function::load_state(&ident)?);
    }
    api.rust_wrapper()
}
