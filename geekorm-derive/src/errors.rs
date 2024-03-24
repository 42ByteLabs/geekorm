use thiserror::Error;

/// To Tokens Error
pub(crate) trait ToTokensError {
    fn to_tokens_error<D: std::fmt::Display>(&self, msg: D) -> syn::Error;
}

impl<T: quote::ToTokens> ToTokensError for T {
    fn to_tokens_error<D: std::fmt::Display>(&self, msg: D) -> syn::Error {
        syn::Error::new_spanned(self.to_token_stream(), msg)
    }
}
