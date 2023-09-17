macro_rules! format_err {
    (call_site, $fmt:literal $(, $any:expr)* $(,)?) => {
        syn::Error::new(Span::call_site(), format!($fmt, $($any)*))
    };
    (span = $span:expr, $fmt:literal $(, $any:expr)* $(,)?) => {
        syn::Error::new($span, format!($fmt, $($any)*))
    };
    ($tokens:expr, $fmt:literal $(, $any:expr)* $(,)?) => {
        syn::Error::new_spanned($tokens, format!($fmt, $($any)*))
    };
}

macro_rules! err {
    ($($arg:expr),* $(,)?) => {
        return Err(crate::errors::format_err!($($arg),*))
    };
}

pub(crate) use {err, format_err};
