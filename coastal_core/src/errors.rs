macro_rules! format_err {
    ($fmt:literal $(, $any:expr)* $(,)?) => {
        syn::Error::new(Span::call_site(), format!($fmt, $($any)*))
    };
    (@$tokens:expr, $fmt:literal $(, $any:expr)* $(,)?) => {
        syn::Error::new_spanned($tokens, format!($fmt, $($any)*))
    };
}

pub(crate) use format_err;
