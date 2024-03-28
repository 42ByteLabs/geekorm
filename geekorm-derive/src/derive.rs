use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use geekorm_core::{
    ColumnType, ColumnTypeOptions, Columns, ForeignKey, PrimaryKey, Table, TableBuilder,
};
use syn::{GenericArgument, Ident, Type, TypePath};

#[cfg(feature = "uuid")]
use uuid::Uuid;

#[cfg(feature = "chrono")]
use chrono::DateTime;

use crate::internal::TableState;

#[derive(Debug, Clone)]
pub(crate) struct TableDerive {
    pub name: String,
    pub columns: ColumnsDerive,
}

impl ToTokens for TableDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let columns = &self.columns;
        tokens.extend(quote! {
            geekorm::Table {
                name: String::from(#name),
                columns: #columns
            }
        });
    }
}

impl From<TableDerive> for Table {
    fn from(value: TableDerive) -> Self {
        Table {
            name: value.name,
            columns: value.columns.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ColumnsDerive {
    pub(crate) columns: Vec<ColumnDerive>,
}

impl ColumnsDerive {
    pub(crate) fn get_primary_key(&self) -> Option<ColumnDerive> {
        self.columns
            .iter()
            .filter_map(|col| {
                if let ColumnTypeDerive::Identifier(_) = &col.coltype {
                    Some(col.clone())
                } else {
                    None
                }
            })
            .next()
    }

    pub(crate) fn get_foreign_keys(&self) -> Vec<ColumnDerive> {
        self.columns
            .iter()
            .filter_map(|c| {
                if let ColumnTypeDerive::ForeignKey(_) = &c.coltype {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convert the columns into a list of parameters for a function
    pub(crate) fn to_params(&self) -> TokenStream {
        let columns = self.columns.iter().map(|c| c.to_params()).filter_map(|c| c);
        quote! {
            #(#columns),*
        }
    }

    /// Creates a new instance of the struct and passes in the columns
    pub(crate) fn to_self(&self) -> TokenStream {
        let columns = self.columns.iter().map(|c| c.to_self());
        quote! {
            Self {
                #(#columns),*
            }
        }
    }
}

impl Iterator for ColumnsDerive {
    type Item = ColumnDerive;

    fn next(&mut self) -> Option<Self::Item> {
        self.columns.pop()
    }
}

impl ToTokens for ColumnsDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let columns = &self.columns;
        tokens.extend(quote! {
            geekorm::Columns {
                columns: vec![
                    #(#columns ),*
                ]
            }
        })
    }
}

impl From<ColumnsDerive> for geekorm_core::Columns {
    fn from(value: ColumnsDerive) -> Self {
        geekorm_core::Columns {
            columns: value.columns.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<Vec<ColumnDerive>> for ColumnsDerive {
    fn from(columns: Vec<ColumnDerive>) -> Self {
        ColumnsDerive { columns }
    }
}

#[derive(Clone)]
pub(crate) struct ColumnDerive {
    pub(crate) identifier: Ident,
    pub(crate) itype: Type,

    pub(crate) name: String,
    pub(crate) coltype: ColumnTypeDerive,
}

impl ColumnDerive {
    /// Create a new instance of Column
    pub(crate) fn new(identifier: Ident, itype: Type) -> Self {
        let name = identifier.to_string();
        let coltype = ColumnTypeDerive::try_from(&itype).unwrap();
        ColumnDerive {
            identifier,
            itype,
            name,
            coltype,
        }
    }

    /// Convert the column into a list of parameters for a function
    pub(crate) fn to_params(&self) -> Option<TokenStream> {
        let identifier = &self.identifier;
        let itype = &self.itype;
        // Ignore PrimaryKey / ForeignKey / Option<T>
        if let Type::Path(TypePath { path, .. }) = itype {
            if let Some(segment) = path.segments.first() {
                if segment.ident == "Option" || segment.ident == "PrimaryKey" {
                    return None;
                } else if segment.ident == "ForeignKey" {
                    // Get inner type of ForeignKey
                    let inner_type = match path.segments.first().unwrap().arguments {
                        syn::PathArguments::AngleBracketed(ref args) => args.args.first().unwrap(),
                        _ => panic!("Expected angle bracketed arguments"),
                    };

                    // Make inner_type a reference

                    return Some(quote! {
                        #identifier: &#inner_type
                    });
                }
            }
        }
        Some(quote! {
            #identifier: #itype
        })
    }

    /// Create a new instance of the struct and pass in the column
    pub(crate) fn to_self(&self) -> TokenStream {
        let identifier = &self.identifier;
        if let Type::Path(TypePath { path, .. }) = &self.itype {
            if let Some(segment) = path.segments.first() {
                if segment.ident == "Option" {
                    // Option is always None in new()
                    return quote! {
                        #identifier: None
                    };
                } else if segment.ident == "PrimaryKey" {
                    // Generate a new primary key
                    return quote! {
                        #identifier: geekorm::PrimaryKey::new()
                    };
                } else if segment.ident == "ForeignKey" {
                    // Generate a new foreign key
                    todo!("RIP");
                    // return quote! {
                    //     #identifier: geekorm::ForeignKey::new(#identifier),
                    // };
                }
            }
        }
        quote! {
            #identifier
        }
    }
}

impl Debug for ColumnDerive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnDerive")
            .field("name", &self.name)
            .field("coltype", &self.coltype)
            .finish()
    }
}

impl ToTokens for ColumnDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let coltype = &self.coltype;
        tokens.extend(quote! {
            geekorm::Column::new(
                String::from(#name),
                #coltype
            )
        });
    }
}

impl From<ColumnDerive> for geekorm_core::Column {
    fn from(value: ColumnDerive) -> Self {
        geekorm_core::Column {
            name: value.name,
            column_type: ColumnType::from(value.coltype),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ColumnTypeDerive {
    Identifier(ColumnTypeOptionsDerive),
    Text(ColumnTypeOptionsDerive),
    Integer(ColumnTypeOptionsDerive),
    Boolean(ColumnTypeOptionsDerive),
    ForeignKey(ColumnTypeOptionsDerive),
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
            ColumnTypeDerive::ForeignKey(options) => tokens.extend(quote! {
                geekorm::ColumnType::ForeignKey(#options)
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
            ColumnTypeDerive::Text(options) => geekorm_core::ColumnType::Text(options.into()),
            ColumnTypeDerive::Integer(options) => geekorm_core::ColumnType::Integer(options.into()),
            ColumnTypeDerive::Boolean(options) => geekorm_core::ColumnType::Boolean(options.into()),
            ColumnTypeDerive::ForeignKey(options) => {
                geekorm_core::ColumnType::ForeignKey(options.into())
            }
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

            Ok(match ident.to_string().as_str() {
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

                    ColumnTypeDerive::Identifier(ColumnTypeOptionsDerive {
                        primary_key: true,
                        foreign_key: String::new(),
                        unique: true,
                        not_null: true,
                    })
                }
                "ForeignKey" => {
                    // ForeignKey<Table> (get Table type)
                    let inner_type = match path.path.segments.first().unwrap().arguments {
                        syn::PathArguments::AngleBracketed(ref args) => args.args.first().unwrap(),
                        _ => abort!(ident, "Unsupported ForeignKey type"),
                    };
                    // Table name
                    let inner_type_name = match inner_type {
                        GenericArgument::Type(Type::Path(TypePath { path, .. })) => {
                            path.segments.first().unwrap().ident.to_string()
                        }
                        _ => panic!("Unsupported ForeignKey type"),
                    };

                    let tables = TableState::load_state_file();
                    // TODO(geekmasher): What if the table hasn't been added yet?
                    let table = tables
                        .find_table(inner_type_name.as_str())
                        .unwrap_or_else(|| panic!("Table {} not found", inner_type_name));

                    // Get the primary key of the table or default to "id"
                    let primary_key = table.get_primary_key();

                    let options = ColumnTypeOptionsDerive {
                        primary_key: false,
                        foreign_key: format!("{}::{}", inner_type_name, primary_key),
                        unique: false,
                        not_null: true,
                    };
                    ColumnTypeDerive::ForeignKey(options)
                }
                // Data types
                "String" => ColumnTypeDerive::Text(opts),
                "i32" | "i64" | "u32" | "u64" => ColumnTypeDerive::Integer(opts),
                "bool" => ColumnTypeDerive::Boolean(opts),
                "Option" => {
                    let new_opts = ColumnTypeOptionsDerive {
                        not_null: false,
                        ..opts
                    };

                    // Get the inner type of the Option
                    let inner_type = match path.path.segments.first().unwrap().arguments {
                        syn::PathArguments::AngleBracketed(ref args) => args.args.first().unwrap(),
                        _ => panic!("Unsupported Option type :: {:?}", ident.to_string()),
                    };

                    // Parse the inner type
                    match inner_type {
                        GenericArgument::Type(typ) => parse_path(typ, new_opts)?,
                        _ => panic!("Unsupported Option type :: {:?}", ident.to_string()),
                    }
                }
                #[cfg(feature = "uuid")]
                "Uuid" => ColumnTypeDerive::Text(opts),
                #[cfg(feature = "chrono")]
                "DateTime" => ColumnTypeDerive::Text(opts),
                _ => abort!(ident, "Unsupported column path type"),
            })
        }

        _ => panic!(
            "Unsupported column generic type :: {:?}",
            typ.to_token_stream()
        ),
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
}

impl Default for ColumnTypeOptionsDerive {
    fn default() -> Self {
        ColumnTypeOptionsDerive {
            primary_key: false,
            unique: false,
            not_null: true,
            foreign_key: String::new(),
        }
    }
}

impl ToTokens for ColumnTypeOptionsDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let primary_key = &self.primary_key;
        let foreign_key = &self.foreign_key;
        let unique = &self.unique;
        let not_null = &self.not_null;

        tokens.extend(quote! {
            geekorm::ColumnTypeOptions {
                primary_key: #primary_key,
                unique: #unique,
                not_null: #not_null,
                foreign_key: String::from(#foreign_key),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::derive::{ColumnTypeDerive, ColumnTypeOptionsDerive};
    use proc_macro2::TokenStream;
    use quote::ToTokens;

    #[test]
    fn test_derive_columntype() {
        let column_type = ColumnTypeDerive::Text(ColumnTypeOptionsDerive::default());
    }

    #[test]
    fn test_derive_columntype_options() {
        let column_type_options = ColumnTypeOptionsDerive::default();
    }
}
