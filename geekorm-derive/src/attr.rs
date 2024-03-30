use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, Ident, LitStr, Token,
};

#[derive(Debug, Clone)]
pub(crate) struct GeekAttribute {
    #[allow(dead_code)]
    pub(crate) span: Ident,
    pub(crate) key: Option<GeekAttributeKeys>,
    pub(crate) value: Option<GeekAttributeValue>,
}

#[derive(Debug, Clone)]
pub(crate) enum GeekAttributeKeys {
    ForeignKey,
    Rename,
    Skip,
}

#[derive(Debug, Clone)]
pub(crate) enum GeekAttributeValue {
    String(String),
}

impl GeekAttribute {
    pub(crate) fn parse_all(all_attrs: &[Attribute]) -> Result<Vec<Self>, syn::Error> {
        let mut parsed = Vec::new();
        for attr in all_attrs {
            if attr.path().is_ident("geekorm") {
                // Parse the attribute
                let parsed_attr = attr
                    .parse_args::<GeekAttribute>()
                    .expect("Failed to parse attribute");

                parsed.push(parsed_attr);
            } else {
                continue;
            };
        }
        Ok(parsed)
    }
}

impl Parse for GeekAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        let key: Option<GeekAttributeKeys> = match name_str.as_str() {
            "foreign_key" => Some(GeekAttributeKeys::ForeignKey),
            "rename" => Some(GeekAttributeKeys::Rename),
            "skip" => Some(GeekAttributeKeys::Skip),
            _ => None,
        };

        let value = if input.peek(Token![=]) {
            // `name = value` attributes.
            let _assign_token = input.parse::<Token![=]>()?; // skip '='
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                Some(GeekAttributeValue::String(lit.value()))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            span: name,
            key,
            value,
        })
    }
}