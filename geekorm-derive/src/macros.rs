macro_rules! format_err {
    ($obj:expr, $($format:tt)+) => {{
        #[allow(unused_imports)]
        use $crate::errors::*;
        let msg = format!($($format)+);
        $obj.to_tokens_error(msg)
    }};
}

macro_rules! abort {
    ($obj:expr, $($format:tt)+) => {{
        return Err(format_err!($obj, $($format)+));
    }};
}
