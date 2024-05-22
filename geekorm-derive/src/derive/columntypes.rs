use geekorm_core::{ColumnType, ColumnTypeOptions};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use syn::{GenericArgument, Ident, Type, TypePath};

#[derive(Debug, Clone)]
pub(crate) enum ColumnTypeDerive {
    Identifier(ColumnTypeOptionsDerive),
    ForeignKey(ColumnTypeOptionsDerive),
    OneToMany(ColumnTypeOptionsDerive),
    Text(ColumnTypeOptionsDerive),
    Integer(ColumnTypeOptionsDerive),
    Boolean(ColumnTypeOptionsDerive),
    Blob(ColumnTypeOptionsDerive),
}

impl ToTokens for ColumnTypeDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ColumnTypeDerive::Identifier(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Identifier(#options)
                });
            }
            ColumnTypeDerive::Text(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Text(#options)
                });
            }
            ColumnTypeDerive::Integer(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Integer(#options)
                });
            }
            ColumnTypeDerive::Boolean(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Boolean(#options)
                });
            }
            ColumnTypeDerive::Blob(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Blob(#options)
                });
            }
            ColumnTypeDerive::ForeignKey(options) => tokens.extend(quote! {
                geekorm::ColumnType::ForeignKey(#options)
            }),
            ColumnTypeDerive::OneToMany(options) => tokens.extend(quote! {
                geekorm::ColumnType::OneToMany(#options)
            }),
        }
    }
}

impl From<ColumnTypeDerive> for geekorm_core::ColumnType {
    fn from(coltype: ColumnTypeDerive) -> Self {
        match coltype {
            ColumnTypeDerive::Identifier(options) => {
                geekorm_core::ColumnType::Identifier(options.into())
            }
            ColumnTypeDerive::OneToMany(options) => {
                geekorm_core::ColumnType::OneToMany(options.into())
            }
            ColumnTypeDerive::Text(options) => geekorm_core::ColumnType::Text(options.into()),
            ColumnTypeDerive::Integer(options) => geekorm_core::ColumnType::Integer(options.into()),
            ColumnTypeDerive::Boolean(options) => geekorm_core::ColumnType::Boolean(options.into()),
            ColumnTypeDerive::Blob(options) => geekorm_core::ColumnType::Blob(options.into()),
            ColumnTypeDerive::ForeignKey(options) => {
                geekorm_core::ColumnType::ForeignKey(options.into())
            }
        }
    }
}

impl From<&ColumnType> for ColumnTypeDerive {
    fn from(coltype: &ColumnType) -> Self {
        match coltype {
            geekorm_core::ColumnType::Identifier(options) => {
                ColumnTypeDerive::Identifier(options.into())
            }
            geekorm_core::ColumnType::ForeignKey(options) => {
                ColumnTypeDerive::ForeignKey(options.into())
            }
            geekorm_core::ColumnType::OneToMany(options) => {
                ColumnTypeDerive::OneToMany(options.into())
            }
            geekorm_core::ColumnType::Text(options) => ColumnTypeDerive::Text(options.into()),
            geekorm_core::ColumnType::Integer(options) => ColumnTypeDerive::Integer(options.into()),
            geekorm_core::ColumnType::Boolean(options) => ColumnTypeDerive::Boolean(options.into()),
            geekorm_core::ColumnType::Blob(options) => ColumnTypeDerive::Blob(options.into()),
        }
    }
}

impl TryFrom<&Type> for ColumnTypeDerive {
    type Error = syn::Error;

    fn try_from(ty: &Type) -> Result<Self, Self::Error> {
        parse_path(ty, ColumnTypeOptionsDerive::default())
    }
}

#[allow(unreachable_patterns, unused_variables, non_snake_case)]
fn parse_path(typ: &Type, opts: ColumnTypeOptionsDerive) -> Result<ColumnTypeDerive, syn::Error> {
    match typ {
        Type::Slice(_) => Ok(ColumnTypeDerive::Text(ColumnTypeOptionsDerive::default())),
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

                    Ok(ColumnTypeDerive::Identifier(ColumnTypeOptionsDerive {
                        primary_key: true,
                        foreign_key: String::new(),
                        unique: false,
                        not_null: false,
                        // If the inner type is an integer, auto increment
                        auto_increment: inner_type_name == "Integer",
                    }))
                }
                "PrimaryKeyString" | "PrimaryKeyUuid" => {
                    Ok(ColumnTypeDerive::Identifier(ColumnTypeOptionsDerive {
                        primary_key: true,
                        foreign_key: String::new(),
                        unique: false,
                        not_null: false,
                        auto_increment: false,
                    }))
                }
                "PrimaryKeyInteger" => Ok(ColumnTypeDerive::Identifier(ColumnTypeOptionsDerive {
                    primary_key: true,
                    foreign_key: String::new(),
                    unique: false,
                    not_null: false,
                    auto_increment: true,
                })),
                "ForeignKey" => {
                    let options = ColumnTypeOptionsDerive {
                        primary_key: false,
                        foreign_key: String::from("GeekOrmForeignKey"),
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    };
                    Ok(ColumnTypeDerive::ForeignKey(options))
                }
                // Data types
                "String" => Ok(ColumnTypeDerive::Text(opts)),
                "i32" | "i64" | "u32" | "u64" => Ok(ColumnTypeDerive::Integer(opts)),
                "bool" => Ok(ColumnTypeDerive::Boolean(opts)),
                "Option" => {
                    let new_opts = ColumnTypeOptionsDerive {
                        not_null: false,
                        ..opts
                    };

                    // Get the inner type of the Option
                    let inner_type = match path.path.segments.first().unwrap().arguments {
                        syn::PathArguments::AngleBracketed(ref args) => args.args.first().unwrap(),
                        _ => return Err(syn::Error::new_spanned(typ, "Unsupported Option type")),
                    };

                    // Parse the inner type
                    match inner_type {
                        GenericArgument::Type(typ) => parse_path(typ, new_opts),
                        _ => Err(syn::Error::new_spanned(typ, "Unsupported Option type")),
                    }
                }
                "Vec" => Ok(ColumnTypeDerive::Blob(opts)),
                #[cfg(feature = "uuid")]
                "Uuid" => Ok(ColumnTypeDerive::Text(opts)),
                #[cfg(feature = "chrono")]
                "DateTime" => Ok(ColumnTypeDerive::Text(opts)),
                // TODO(geekmasher): Remove this
                _ => Err(syn::Error::new_spanned(
                    ident,
                    "Unsupported column path type",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(typ, "Unsupported column type")),
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct ColumnTypeOptionsDerive {
    pub(crate) primary_key: bool,
    // TableName::ColumnKey
    pub(crate) foreign_key: String,
    pub(crate) unique: bool,
    pub(crate) not_null: bool,
    pub(crate) auto_increment: bool,
}

impl Default for ColumnTypeOptionsDerive {
    fn default() -> Self {
        ColumnTypeOptionsDerive {
            primary_key: false,
            unique: false,
            not_null: true,
            foreign_key: String::new(),
            auto_increment: false,
        }
    }
}

impl ToTokens for ColumnTypeOptionsDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let primary_key = &self.primary_key;
        let foreign_key = &self.foreign_key;
        let unique = &self.unique;
        let not_null = &self.not_null;
        let auto_increment = &self.auto_increment;

        tokens.extend(quote! {
            geekorm::ColumnTypeOptions {
                primary_key: #primary_key,
                unique: #unique,
                not_null: #not_null,
                foreign_key: String::from(#foreign_key),
                auto_increment: #auto_increment,
            }
        });
    }
}

impl From<ColumnTypeOptionsDerive> for geekorm_core::ColumnTypeOptions {
    fn from(opts: ColumnTypeOptionsDerive) -> geekorm_core::ColumnTypeOptions {
        geekorm_core::ColumnTypeOptions {
            primary_key: opts.primary_key,
            foreign_key: opts.foreign_key,
            unique: opts.unique,
            not_null: opts.not_null,
            auto_increment: opts.auto_increment,
        }
    }
}

impl From<&ColumnTypeOptions> for ColumnTypeOptionsDerive {
    fn from(opts: &ColumnTypeOptions) -> ColumnTypeOptionsDerive {
        ColumnTypeOptionsDerive {
            primary_key: opts.primary_key,
            foreign_key: opts.foreign_key.clone(),
            unique: opts.unique,
            not_null: opts.not_null,
            auto_increment: opts.auto_increment,
        }
    }
}
