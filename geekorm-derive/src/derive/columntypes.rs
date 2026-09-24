use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{GenericArgument, Ident, Type, TypePath};

use super::ColumnDerive;

#[derive(Debug, Clone)]
pub(crate) enum ColumnTypeDerive {
    Text,
    Integer,
    Boolean,
    Blob,
    ForeignKey,
}

impl ToTokens for ColumnTypeDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ColumnTypeDerive::Text => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Text
                });
            }
            ColumnTypeDerive::Integer => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Integer
                });
            }
            ColumnTypeDerive::Boolean => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Boolean
                });
            }
            ColumnTypeDerive::Blob => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Blob
                });
            }
            ColumnTypeDerive::ForeignKey => tokens.extend(quote! {
                geekorm::ColumnType::ForeignKey
            }),
        }
    }
}

impl From<ColumnTypeDerive> for geekorm_sql::ColumnType {
    fn from(coltype: ColumnTypeDerive) -> Self {
        match coltype {
            ColumnTypeDerive::Text => geekorm_sql::ColumnType::Text,
            ColumnTypeDerive::Integer => geekorm_sql::ColumnType::Integer,
            ColumnTypeDerive::Boolean => geekorm_sql::ColumnType::Boolean,
            ColumnTypeDerive::Blob => geekorm_sql::ColumnType::Blob,
            ColumnTypeDerive::ForeignKey => geekorm_sql::ColumnType::ForeignKey,
        }
    }
}

/// This function parses and creates the correct Column Type and Options based on the information
/// passed into it via the derive marco.
///
/// HACK: This is a hack and needs massive improvement
#[allow(unreachable_patterns, unused_variables, non_snake_case)]
pub(crate) fn parse_path(
    typ: &Type,
) -> Result<(ColumnTypeDerive, ColumnOptionsDerive), syn::Error> {
    match typ {
        Type::Slice(_) => Ok((ColumnTypeDerive::Text, ColumnOptionsDerive::default())),
        Type::Path(path) => {
            let ident = path.path.segments.first().unwrap().ident.clone();

            let ident_name = ident.to_string();

            match ident_name.as_str() {
                // GeekORM types
                "PrimaryKey" => {
                    let inner_type = match path.path.segments.first().unwrap().arguments {
                        syn::PathArguments::AngleBracketed(ref args) => args.args.first().unwrap(),
                        _ => abort!(ident, "Unsupported PrimaryKey type"),
                    };
                    let inner_type_name = match inner_type {
                        GenericArgument::Type(Type::Path(TypePath { path, .. })) => {
                            path.segments.first().unwrap().ident.to_string()
                        }
                        _ => panic!("Unsupported PrimaryKey type"),
                    };

                    // TODO: Bit of a temp hack
                    let ctype = if inner_type_name == "Integer" {
                        ColumnTypeDerive::Integer
                    } else {
                        ColumnTypeDerive::Text
                    };

                    Ok((
                        ctype,
                        ColumnOptionsDerive {
                            primary_key: true,
                            unique: false,
                            not_null: false,
                            // If the inner type is an integer, auto increment
                            auto_increment: inner_type_name == "Integer",
                        },
                    ))
                }
                "PrimaryKeyString" | "PrimaryKeyUuid" => Ok((
                    ColumnTypeDerive::Text,
                    ColumnOptionsDerive {
                        primary_key: true,
                        unique: false,
                        not_null: false,
                        auto_increment: false,
                    },
                )),
                "PrimaryKeyInteger" => Ok((
                    ColumnTypeDerive::Integer,
                    ColumnOptionsDerive {
                        primary_key: true,
                        unique: false,
                        not_null: false,
                        auto_increment: true,
                    },
                )),
                "ForeignKey" => Ok((
                    ColumnTypeDerive::ForeignKey,
                    ColumnOptionsDerive {
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                )),
                // Data types
                "String" => Ok((ColumnTypeDerive::Text, ColumnOptionsDerive::default())),
                "i32" | "i64" | "u32" | "u64" => {
                    Ok((ColumnTypeDerive::Integer, ColumnOptionsDerive::default()))
                }
                "bool" => Ok((ColumnTypeDerive::Boolean, ColumnOptionsDerive::default())),
                "Option" => {
                    // Get the inner type of the Option
                    let inner_type = match path.path.segments.first().unwrap().arguments {
                        syn::PathArguments::AngleBracketed(ref args) => args.args.first().unwrap(),
                        _ => {
                            return Err(syn::Error::new_spanned(typ, "Unsupported Option type"));
                        }
                    };

                    // Parse the inner type
                    match inner_type {
                        GenericArgument::Type(typ) => {
                            let mut inner = parse_path(typ)?;
                            // set as nullable
                            inner.1.set_notnull(false);

                            Ok((inner.0, inner.1))
                        }
                        _ => Err(syn::Error::new_spanned(typ, "Unsupported Option type")),
                    }
                }
                "Vec" => Ok((ColumnTypeDerive::Blob, ColumnOptionsDerive::default())),
                #[cfg(feature = "uuid")]
                "Uuid" => Ok((ColumnTypeDerive::Text, ColumnOptionsDerive::default())),
                #[cfg(feature = "chrono")]
                "chrono" | "DateTime" => {
                    Ok((ColumnTypeDerive::Text, ColumnOptionsDerive::default()))
                }

                // Default to blob
                _ => Ok((ColumnTypeDerive::Blob, ColumnOptionsDerive::default())),
            }
        }
        _ => Err(syn::Error::new_spanned(typ, "Unsupported column type")),
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct ColumnOptionsDerive {
    pub(crate) primary_key: bool,
    /// Column is unique
    pub(crate) unique: bool,
    /// Column is not null
    pub(crate) not_null: bool,
    /// Column is auto increment
    pub(crate) auto_increment: bool,
}

impl ColumnOptionsDerive {
    /// Set Unique
    pub fn set_unique(&mut self, unique: bool) {
        self.unique = unique;
    }
    /// Set Not Null
    pub fn set_notnull(&mut self, notnull: bool) {
        self.not_null = notnull;
    }
    /// Set Auto Increment
    pub fn set_auto_increment(&mut self, auto_increment: bool) {
        self.auto_increment = auto_increment;
    }
}

impl Default for ColumnOptionsDerive {
    fn default() -> Self {
        ColumnOptionsDerive {
            primary_key: false,
            unique: false,
            not_null: true,
            auto_increment: false,
        }
    }
}

impl ToTokens for ColumnOptionsDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let primary_key = &self.primary_key;
        let unique = &self.unique;
        let not_null = &self.not_null;
        let auto_increment = &self.auto_increment;

        tokens.extend(quote! {
            geekorm::ColumnOptions {
                primary_key: #primary_key,
                unique: #unique,
                not_null: #not_null,
                auto_increment: #auto_increment,
            }
        });
    }
}

impl From<ColumnOptionsDerive> for geekorm_core::ColumnTypeOptions {
    fn from(opts: ColumnOptionsDerive) -> geekorm_core::ColumnTypeOptions {
        geekorm_core::ColumnOptions {
            primary_key: opts.primary_key,
            unique: opts.unique,
            not_null: opts.not_null,
            auto_increment: opts.auto_increment,
        }
    }
}
