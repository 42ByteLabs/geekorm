use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use super::GeekAttribute;
use crate::attr::GeekAttributeValue;

// impl From<&UserRole> for geekorm::Value {
//     fn from(value: &UserRole) -> Self {
//         match value {
//             UserRole::Admin => geekorm::Value::Integer(0),
//             UserRole::User => geekorm::Value::Integer(1),
//         }
//     }
// }
pub(crate) fn generate_from_value(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    generics: &syn::Generics,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();
    let mut from_value_stream = TokenStream::new();

    for variant in variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Only unit variants are supported",
            ));
        }
        if variant.discriminant.is_some() {
            return Err(syn::Error::new(
                variant.span(),
                "Discriminant values are not supported",
            ));
        }

        let attributes = GeekAttribute::parse_all(&variant.attrs)?;

        let variant_ident = variant.ident.clone();

        // Support `key` or `rename` attribute
        let variant_str = if let Some(attr) = attributes
            .iter()
            .find(|&attr| attr.key == Some(crate::attr::GeekAttributeKeys::Key))
        {
            if let Some(GeekAttributeValue::String(value)) = &attr.value {
                syn::LitStr::new(value, value.span())
            } else if let Some(GeekAttributeValue::Int(value)) = &attr.value {
                syn::LitStr::new(value.to_string().as_str(), value.span())
            } else {
                return Err(syn::Error::new(
                    attr.span.span(),
                    "Expected string or int value for `rename` attribute",
                ));
            }
        } else {
            // TODO: Handle r# prefix better
            let variant_string = variant_ident.to_string().replace("r#", "");
            syn::LitStr::new(&variant_string, variant.span())
        };

        stream.extend(quote! {
            #ident::#variant_ident => ::geekorm::Value::Text(value.to_string()),
        });
        from_value_stream.extend(quote! {
            ::geekorm::Value::Text(ref s) if s == #variant_str => #ident::#variant_ident,
        });
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics From<#ident #ty_generics> for ::geekorm::Value {
            fn from(value: #ident #ty_generics) -> Self {
                match value {
                    #stream
                    _ => panic!("Unknown value"),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#ident #ty_generics> for ::geekorm::Value {
            fn from(value: &#ident #ty_generics) -> Self {
                match value {
                    #stream
                    _ => panic!("Unknown value"),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<::geekorm::Value> for #ident #ty_generics {
            fn from(value: geekorm::Value) -> Self {
                match value {
                    #from_value_stream
                    _ => panic!("Unknown value"),
                }
            }
        }
    })
}

/// Generating ToString / Display implementations for the enum
///
/// ```rust
/// # use geekorm::prelude::*;
///
/// # #[derive(Eq, PartialEq, Debug)]
/// #[derive(Data, Default, Clone)]
/// enum UserRole {
///     #[geekorm(key = "Administrator")]
///     Admin,
///     #[geekorm(key = "AdminModerator")]
///     Moderator,
///     User,
///     #[default]
///     Guest,
/// }
///
/// // The parsing is case-sensitive
/// let user_type = UserRole::from("Administrator");
/// # assert_eq!(user_type, UserRole::Admin);
/// # assert_eq!(user_type.to_string(), "Administrator".to_string());
///
/// let moderator = UserRole::Moderator;
/// # assert_eq!(moderator.to_string(), "AdminModerator".to_string());
/// # let mod_value = geekorm::Value::Text(moderator.to_string());
/// # assert_eq!(UserRole::from(mod_value), UserRole::Moderator);
///
/// let unknown = UserRole::from("unknown");
/// // This will use the default value
/// assert_eq!(unknown, UserRole::Guest);
/// ```
///
/// ### Disabling Features
///
/// ```rust
/// # use geekorm::prelude::*;
///
/// # #[derive(Eq, PartialEq, Debug)]
/// #[derive(Data, Default, Clone)]
/// #[geekorm(disable = "from_string")]     // Disable parsing from string
/// enum UserRole {
///     Admin,
///     Moderator,
///     User,
///     #[default]
///     Guest,
/// }
///
/// // I can now implement the parsing myself
/// impl From<&str> for UserRole {
///     fn from(value: &str) -> Self {
///         match value.to_string().to_lowercase().as_str() {
///             "admin" | "root" => UserRole::Admin,
///             "moderator" | "mod" => UserRole::Moderator,
///             "user" => UserRole::User,
///             "guest" => UserRole::Guest,
///             // Defaults
///             _ => UserRole::Guest,
///         }
///     }
/// }
///
/// impl From<String> for UserRole {
///     fn from(value: String) -> Self {
///         Self::from(value.as_str())
///     }
/// }
///
/// // Try our parsing
/// let user = UserRole::from("mod");
/// # assert_eq!(user, UserRole::Moderator);
/// # let user_string = UserRole::from(String::from("mod"));
/// # assert_eq!(user_string, UserRole::Moderator);
///
/// ```
pub(crate) fn generate_strings(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    _generics: &syn::Generics,
    attributes: &[GeekAttribute],
) -> Result<TokenStream, syn::Error> {
    let mut stream = TokenStream::new();
    let mut str_to = TokenStream::new();

    let from_lowercase: bool = attributes.iter().any(|attr| {
        attr.key == Some(crate::attr::GeekAttributeKeys::FromString)
            && attr.value == Some(GeekAttributeValue::String("lowercase".to_string()))
    });
    let to_lowercase: bool = attributes.iter().any(|attr| {
        attr.key == Some(crate::attr::GeekAttributeKeys::ToString)
            && attr.value == Some(GeekAttributeValue::String("lowercase".to_string()))
    });
    // This is an advanced feature to disable the parsing from strings
    let disabled_from_strings = attributes.iter().any(|attr| {
        attr.key == Some(crate::attr::GeekAttributeKeys::Disable)
            && attr.value == Some(GeekAttributeValue::String("from_str".to_string()))
    });

    for variant in variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Only unit variants are supported",
            ));
        }
        if variant.discriminant.is_some() {
            return Err(syn::Error::new(
                variant.span(),
                "Discriminant values are not supported",
            ));
        }

        let attrs = GeekAttribute::parse_all(&variant.attrs)?;

        let variant_ident = variant.ident.clone();

        // Support `key` or `rename` attribute
        // We do not lowercase the value here, as we want to keep the original
        let variant_str = if let Some(attr) = attrs
            .iter()
            .find(|&attr| attr.key == Some(crate::attr::GeekAttributeKeys::Key))
        {
            if let Some(GeekAttributeValue::String(value)) = &attr.value {
                syn::LitStr::new(&value, value.span())
            } else if let Some(GeekAttributeValue::Int(value)) = &attr.value {
                syn::LitStr::new(value.to_string().as_str(), value.span())
            } else {
                return Err(syn::Error::new(
                    attr.span.span(),
                    "Expected string or int value for `rename` attribute",
                ));
            }
        } else {
            // TODO: Handle r# prefix better
            let mut variant_string = variant_ident.to_string().replace("r#", "");
            if to_lowercase {
                variant_string = variant_string.to_lowercase();
            }
            syn::LitStr::new(&variant_string, variant.span())
        };

        // The list of &str values to match against
        let mut variants: Vec<syn::LitStr> = vec![variant_str.clone()];

        if let Some(aliases) = attrs
            .iter()
            .find(|&attr| attr.key == Some(crate::attr::GeekAttributeKeys::Aliases))
        {
            match &aliases.value {
                Some(GeekAttributeValue::String(value)) => {
                    variants.push(syn::LitStr::new(&value, aliases.span.span()));
                }
                Some(GeekAttributeValue::List(values)) => {
                    for value in values {
                        variants.push(syn::LitStr::new(&value, value.span()));
                    }
                }
                _ => {}
            }
        }

        stream.extend(quote! {
            #ident::#variant_ident => String::from(#variant_str),
        });
        // key | aliases*
        str_to.extend(quote! {
            #(#variants)|* => #ident::#variant_ident,
        });
    }

    // The parsing is case-sensitive
    let str_from = if from_lowercase {
        quote! {
            match s.to_lowercase().as_str() {
                #str_to
                _ => return Err(::geekorm::Error::UnknownVariant(s.to_string())),
            }
        }
    } else {
        quote! {
            match s {
                #str_to
                _ => return Err(::geekorm::Error::UnknownVariant(s.to_string())),
            }
        }
    };

    let strings_tokens = if !disabled_from_strings {
        quote! {
            #[automatically_derived]
            impl ::std::str::FromStr for #ident {
                type Err = ::geekorm::Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Ok( #str_from )
                }
            }
            #[automatically_derived]
            impl From<&str> for #ident
            where
                Self: Default
            {
                fn from(value: &str) -> Self {
                    use ::std::str::FromStr;
                    Self::from_str(value).unwrap_or_default()
                }
            }
            #[automatically_derived]
            impl From<String> for #ident
            where
                Self: Default
            {
                fn from(value: String) -> Self {
                    use ::std::str::FromStr;
                    Self::from_str(value.as_str()).unwrap_or_default()
                }
            }
            #[automatically_derived]
            impl From<&String> for #ident
            where
                Self: Default
            {
                fn from(value: &String) -> Self {
                    use ::std::str::FromStr;
                    Self::from_str(value.as_str()).unwrap_or_default()
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #strings_tokens

        #[automatically_derived]
        impl ::std::fmt::Display for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        #stream
                    }
                )
            }
        }
    })
}

pub(crate) fn generate_serde(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    generics: &syn::Generics,
) -> Result<TokenStream, syn::Error> {
    let (_impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();

    stream.extend(quote! {
        #[automatically_derived]
        impl ::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                ::geekorm::Value::from(self).serialize(serializer)
            }
        }
    });

    let mut tokens = TokenStream::new();

    for variant in variants.iter() {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Only unit variants are supported",
            ));
        }
        if variant.discriminant.is_some() {
            return Err(syn::Error::new(
                variant.span(),
                "Discriminant values are not supported",
            ));
        }

        let variant_ident = variant.ident.clone();
        let variant_string = variant_ident.to_string().replace("r#", "");
        let variant_str = syn::LitStr::new(&variant_string, variant.span());

        tokens.extend(quote! {
            #variant_str => Ok(#ident::#variant_ident),
        });
    }

    stream.extend(quote! {
        #[automatically_derived]
        impl<'de> ::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                use ::std::str::FromStr;
                Self::from_str(String::deserialize(deserializer)?.as_str())
                    .map_err(::serde::de::Error::custom)
            }
        }
    });

    Ok(stream)
}
