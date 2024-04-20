use proc_macro2::Span;
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
    pub(crate) value_span: Option<Span>,
}

#[derive(Debug, Clone)]
pub(crate) enum GeekAttributeKeys {
    Skip,
    Rename,
    Default,
    AutoIncrement,
    ForeignKey,
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
                let parsed_attr = match attr.parse_args::<GeekAttribute>() {
                    Ok(parsed_attr) => parsed_attr,
                    Err(e) => return Err(e),
                };

                parsed_attr.validate()?;

                parsed.push(parsed_attr);
            } else {
                continue;
            };
        }
        Ok(parsed)
    }

    #[allow(irrefutable_let_patterns)]
    pub(crate) fn validate(&self) -> Result<(), syn::Error> {
        match self.key {
            // Validate the `foreign_key` attribute
            Some(GeekAttributeKeys::ForeignKey) => {
                if let Some(value) = &self.value {
                    if let GeekAttributeValue::String(content) = value {
                        if let Some((_, _)) = content.split_once('.') {
                            // TODO(geekmasher): Lookup and validate the table.column
                            Ok(())
                        } else {
                            Err(syn::Error::new(
                                self.span.span(),
                                "The `foreign_key` attribute requires a table.column value",
                            ))
                        }
                    } else {
                        Err(syn::Error::new(
                            self.span.span(),
                            "The `foreign_key` attribute requires a string value",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `foreign_key` attribute requires a value",
                    ))
                }
            }
            _ => Ok(()),
        }
    }
}

impl Parse for GeekAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        let key: Option<GeekAttributeKeys> = match name_str.as_str() {
            "skip" => Some(GeekAttributeKeys::Skip),
            "rename" => Some(GeekAttributeKeys::Rename),
            "default" => Some(GeekAttributeKeys::Default),
            "auto_increment" => Some(GeekAttributeKeys::AutoIncrement),
            "foreign_key" => Some(GeekAttributeKeys::ForeignKey),
            _ => None,
        };

        let mut value_span: Option<Span> = None;

        let value = if input.peek(Token![=]) {
            // `name = value` attributes.
            let _assign_token = input.parse::<Token![=]>()?; // skip '='
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                value_span = Some(lit.span());

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
            value_span,
        })
    }
}
