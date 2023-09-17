use attr_macro::attr_macro_coastal;
use impl_macro::macro_coastal_impl;
use proc_macro2::TokenStream;

mod api;
mod attr_macro;
mod errors;
mod impl_macro;
mod state;
mod types;

#[proc_macro_attribute]
pub fn coastal(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match attr_macro_coastal(TokenStream::from(attr), TokenStream::from(input)) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn coastal_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match macro_coastal_impl(TokenStream::from(input)) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
