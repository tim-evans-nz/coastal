use proc_macro2::TokenStream;

use coastal_core::macros;

#[proc_macro_attribute]
pub fn coast(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match macros::coast(TokenStream::from(attr), TokenStream::from(input)) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn api(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match macros::api(TokenStream::from(input)) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
