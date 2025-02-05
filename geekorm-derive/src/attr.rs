//! Geek Attributes for the derive macro
//!
//! # Samples
//!
//! ```rust
//! use geekorm::prelude::*;
//!
//! #[derive(Table, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
//! struct Users {
//!     #[geekorm(primary_key, auto_increment)]
//!     id: PrimaryKey<i32>,
//!     /// Rename the field for the table
//!     #[geekorm(rename = "full_name")]
//!     name: String,
//!
//!     age: i32,
//!
//!     occupation: String,
//!     /// Random value
//! #   #[cfg(feature = "rand")]
//!     #[geekorm(unique, rand, rand_length = "42", rand_prefix = "gorm_")]
//!     session: String,
//!     /// Datetime using chrono
//! #   #[cfg(feature = "chrono")]
//!     #[geekorm(new = "chrono::Utc::now()")]
//!     created_at: chrono::DateTime<chrono::Utc>,
//! }
//!
//! #[derive(Table, Debug, Clone, serde::Serialize, serde::Deserialize)]
//! struct Posts {
//!     #[geekorm(primary_key, auto_increment)]
//!     id: PrimaryKeyInteger,
//!     #[geekorm(not_null)]
//!     title: String,
//!     #[geekorm(foreign_key = "Users.id")]
//!     author: ForeignKey<i32, Users>,
//! }
//!
//! # fn main() {
//!     let user = Users::new(
//!         "geekmasher",
//!         42,
//!         "Software Engineer",
//!     );
//!     let post = Posts::new(
//!         "Why I love Rust",
//!         user.id
//!     );
//! # }
//! ```
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::AnyDelimiter, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Bracket, Comma},
    Attribute, Ident, LitBool, LitInt, LitStr, Token,
};

#[derive(Debug, Clone)]
pub(crate) struct GeekAttribute {
    #[allow(dead_code)]
    pub(crate) span: Ident,
    pub(crate) key: Option<GeekAttributeKeys>,
    pub(crate) value: Option<GeekAttributeValue>,
    pub(crate) value_span: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum GeekAttributeKeys {
    /// Rename the field for the table
    Rename,
    /// Database Name
    Database,
    /// ToString
    ToString,
    FromString,
    /// Key
    Key,
    /// Unique value
    Unique,
    /// New Constructor
    New,
    /// Primary Key
    PrimaryKey,
    /// Auto Increment
    AutoIncrement,
    /// Not Null
    NotNull,
    /// Foreign Key
    ForeignKey,
    /// Aliases
    Aliases,
    /// Random value
    Rand,
    RandLength,
    RandPrefix,
    RandEnv,
    /// Hash / Password
    Hash,
    HashAlgorithm,
    /// Searchable
    Searchable,
    /// On Actions
    OnValidate,
    OnUpdate,
    OnSave,
    /// Skip this field
    Skip,
    /// Disable features
    Disable,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum GeekAttributeValue {
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<String>),
}

const TO_STRING_KEYS: [&str; 1] = ["lowercase"];

impl GeekAttribute {
    pub(crate) fn parse_all(all_attrs: &[Attribute]) -> Result<Vec<Self>, syn::Error> {
        let mut parsed = Vec::new();
        for attribute in all_attrs {
            if attribute.path().is_ident("geekorm") {
                for attr in attribute
                    .parse_args_with(Punctuated::<GeekAttribute, Token![,]>::parse_terminated)?
                {
                    // Validate the attribute before adding it to the parsed list
                    attr.validate()?;
                    parsed.push(attr);
                }
            } else {
                continue;
            };
        }
        Ok(parsed)
    }

    #[allow(irrefutable_let_patterns)]
    pub(crate) fn validate(&self) -> Result<(), syn::Error> {
        match self.key {
            // Requires: The `primary_key` attribute does not require a value
            Some(GeekAttributeKeys::PrimaryKey) => {
                if self.value.is_some() {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `primary_key` attribute does not require a value",
                    ))
                } else {
                    Ok(())
                }
            }
            Some(GeekAttributeKeys::OnUpdate) => {
                if let Some(GeekAttributeValue::String(_)) = &self.value {
                    Ok(())
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `update` attribute requires a String value",
                    ))
                }
            }
            Some(GeekAttributeKeys::OnSave) => {
                if let Some(GeekAttributeValue::String(_)) = &self.value {
                    Ok(())
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `save` attribute requires a String value",
                    ))
                }
            }
            Some(GeekAttributeKeys::New) => {
                // Requires: The `new` attribute requires a string or bool value
                if let Some(value) = &self.value {
                    if let GeekAttributeValue::String(_) = value {
                        Ok(())
                    } else if let GeekAttributeValue::Bool(_) = value {
                        Ok(())
                    } else {
                        Err(syn::Error::new(
                            self.span.span(),
                            "The `new` attribute requires a string value",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `new` attribute requires a value",
                    ))
                }
            }
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
            Some(GeekAttributeKeys::HashAlgorithm) => {
                if let Some(value) = &self.value {
                    if let GeekAttributeValue::String(content) = value {
                        if geekorm_core::utils::crypto::HashingAlgorithm::try_from(content).is_ok()
                        {
                            Ok(())
                        } else {
                            Err(syn::Error::new(
                                self.value_span.unwrap_or_else(|| self.span.span()),
                                "The `hash_algorithm` attribute requires a supported hashing algorithm",
                            ))
                        }
                    } else {
                        Err(syn::Error::new(
                            self.span.span(),
                            "The `hash_algorithm` attribute requires a string value",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `hash_algorithm` attribute requires a value",
                    ))
                }
            }
            Some(GeekAttributeKeys::Searchable) => {
                if self.value.is_some() {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `searchable` attribute does not require a value",
                    ))
                } else {
                    Ok(())
                }
            }
            Some(GeekAttributeKeys::Key) => {
                if self.value.is_none() {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `key` attribute requires a string or int value",
                    ))
                } else {
                    Ok(())
                }
            }
            Some(GeekAttributeKeys::ToString) => {
                if let Some(value) = &self.value {
                    if let GeekAttributeValue::String(value_str) = value {
                        if TO_STRING_KEYS.contains(&value_str.as_str()) {
                            Ok(())
                        } else {
                            Err(syn::Error::new(
                                self.span.span(),
                                "The `to_string` attribute only supports `lowercase`",
                            ))
                        }
                    } else {
                        Err(syn::Error::new(
                            self.span.span(),
                            "The `to_string` attribute requires a string value",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `to_string` attribute requires a value",
                    ))
                }
            }
            Some(GeekAttributeKeys::Disable) => {
                if let Some(value) = &self.value {
                    if let GeekAttributeValue::List(_) = value {
                        Ok(())
                    } else {
                        Err(syn::Error::new(
                            self.span.span(),
                            "The `disable` attribute requires a list of strings",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        self.span.span(),
                        "The `key` attribute requires a string or int value",
                    ))
                }
            }
            _ => Ok(()),
        }
    }
}

const VEC_KEYS: [&str; 2] = ["aliases", "disable"];

impl Parse for GeekAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        let key: Option<GeekAttributeKeys> = match name_str.as_str() {
            "skip" => Some(GeekAttributeKeys::Skip),
            "database" | "db" => Some(GeekAttributeKeys::Database),
            "disable" => Some(GeekAttributeKeys::Disable),
            "rename" => Some(GeekAttributeKeys::Rename),
            "to_str" | "to_string" => Some(GeekAttributeKeys::ToString),
            "from_str" | "from_string" => Some(GeekAttributeKeys::FromString),
            "key" | "name" => Some(GeekAttributeKeys::Key),
            "aliases" => Some(GeekAttributeKeys::Aliases),
            // Primary Keys
            "primary_key" => Some(GeekAttributeKeys::PrimaryKey),
            "auto_increment" => Some(GeekAttributeKeys::AutoIncrement),
            "not_null" => Some(GeekAttributeKeys::NotNull),
            "unique" => Some(GeekAttributeKeys::Unique),
            // Foreign Key
            "foreign_key" => Some(GeekAttributeKeys::ForeignKey),
            // Functions on action
            "validate" | "on_validate" => Some(GeekAttributeKeys::OnValidate),
            "update" | "on_update" | "on_update_write" => Some(GeekAttributeKeys::OnUpdate),
            "save" | "on_save" | "on_save_write" => Some(GeekAttributeKeys::OnSave),

            // New Constructor
            "new" => match cfg!(feature = "new") {
                true => Some(GeekAttributeKeys::New),
                false => {
                    return Err(syn::Error::new(
                        name.span(),
                        "The `new` attribute requires the `new` feature to be enabled",
                    ))
                }
            },
            // Random value feature
            "rand" => match cfg!(feature = "rand") {
                true => Some(GeekAttributeKeys::Rand),
                false => {
                    return Err(syn::Error::new(
                        name.span(),
                        "The `rand` attribute requires the `rand` feature to be enabled",
                    ))
                }
            },
            "rand_length" => match cfg!(feature = "rand") {
                true => Some(GeekAttributeKeys::RandLength),
                false => {
                    return Err(syn::Error::new(
                        name.span(),
                        "The `rand_length` attribute requires the `rand` feature to be enabled",
                    ))
                }
            },
            "rand_prefix" => match cfg!(feature = "rand") {
                true => Some(GeekAttributeKeys::RandPrefix),
                false => {
                    return Err(syn::Error::new(
                        name.span(),
                        "The `rand_prefix` attribute requires the `rand` feature to be enabled",
                    ))
                }
            },
            "rand_env" => match cfg!(feature = "rand") {
                true => Some(GeekAttributeKeys::RandEnv),
                false => {
                    return Err(syn::Error::new(
                        name.span(),
                        "The `rand_env` attribute requires the `rand` feature to be enabled",
                    ))
                }
            },
            "hash" | "password" => match cfg!(feature = "hash") {
                true => Some(GeekAttributeKeys::Hash),
                false => return Err(syn::Error::new(
                    name.span(),
                    "The `hash` or `password` attribute requires the `hash` feature to be enabled",
                )),
            },
            "hash_algorithm" => {
                match cfg!(feature = "hash") {
                    true => Some(GeekAttributeKeys::HashAlgorithm),
                    false => return Err(syn::Error::new(
                        name.span(),
                        "The `hash_algorithm` attribute requires the `hash` feature to be enabled",
                    )),
                }
            }
            "search" | "searchable" => {
                match cfg!(feature = "search") {
                    true => Some(GeekAttributeKeys::Searchable),
                    false => return Err(syn::Error::new(
                        name.span(),
                        "The `searchable` attribute requires the `search` feature to be enabled",
                    )),
                }
            }
            _ => {
                return Err(syn::Error::new(
                    name.span(),
                    format!("Unknown attribute `{}`", name_str),
                ))
            }
        };

        let mut value_span: Option<Span> = None;

        let value = if input.peek(Token![=]) {
            // `name = value` attributes.
            let _assign_token = input.parse::<Token![=]>()?; // skip '='
            if input.peek(LitStr) {
                let lit: LitStr = input.parse()?;
                value_span = Some(lit.span());

                let strings = lit.value();

                if VEC_KEYS.contains(&name_str.as_str()) {
                    Some(GeekAttributeValue::List(
                        strings.split(',').map(|s| s.trim().to_string()).collect(),
                    ))
                } else {
                    Some(GeekAttributeValue::String(strings))
                }
            } else if input.peek(LitInt) {
                let lit: LitInt = input.parse()?;
                value_span = Some(lit.span());

                Some(GeekAttributeValue::Int(lit.base10_parse().unwrap()))
            } else if input.peek(LitBool) {
                let lit: LitBool = input.parse()?;
                value_span = Some(lit.span());

                Some(GeekAttributeValue::Bool(lit.value))
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
