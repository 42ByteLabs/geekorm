use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use geekorm_core::{
    ColumnType, ColumnTypeOptions, Columns, ForeignKey, PrimaryKey, Table, TableBuilder,
};
use syn::{GenericArgument, Ident, Type, TypePath};

#[cfg(feature = "uuid")]
use uuid::Uuid;

#[cfg(feature = "chrono")]
use chrono::DateTime;

use crate::{
    attr::{GeekAttribute, GeekAttributeKeys, GeekAttributeValue},
    internal::TableState,
};

#[derive(Debug, Clone)]
pub(crate) struct TableDerive {
    pub name: String,
    pub columns: ColumnsDerive,
}

impl TableDerive {
    #[allow(irrefutable_let_patterns)]
    pub(crate) fn apply_attributes(&mut self, attributes: &Vec<GeekAttribute>) {
        for attr in attributes {
            if let Some(key) = &attr.key {
                match key {
                    GeekAttributeKeys::Rename => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                self.name = name.to_string();
                            }
                        }
                    }
                    GeekAttributeKeys::Skip | GeekAttributeKeys::ForeignKey => {}
                }
            } else {
                // TODO(geekmasher): Handle this better
            }
        }
    }
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
    pub(crate) attributes: Vec<GeekAttribute>,

    /// Name to be used in the database
    pub(crate) name: String,
    /// Alias to the original struct name
    pub(crate) alias: Option<String>,
    pub(crate) coltype: ColumnTypeDerive,
    pub(crate) skip: bool,
}

impl ColumnDerive {
    /// Create a new instance of Column
    pub(crate) fn new(identifier: Ident, itype: Type, attributes: Vec<GeekAttribute>) -> Self {
        let name = identifier.to_string();
        let coltype = ColumnTypeDerive::try_from(&itype).unwrap();
        let mut col = ColumnDerive {
            identifier,
            itype,
            attributes,
            name,
            coltype,
            alias: None,
            skip: false,
        };
        col.apply_attributes();
        col
    }

    #[allow(irrefutable_let_patterns)]
    pub(crate) fn apply_attributes(&mut self) {
        let attributes = &self.attributes;

        for attr in attributes {
            if let Some(key) = &attr.key {
                match key {
                    GeekAttributeKeys::Skip => {
                        self.skip = true;
                    }
                    GeekAttributeKeys::ForeignKey => {
                        if let Some(value) = &attr.value {
                            if let GeekAttributeValue::String(name) = value {
                                // TODO(geekmasher): Handle this better
                                let (table, column) = name.split_once('.').unwrap_or_else(|| {
                                    panic!("Invalid foreign key format (table.column): {}", name)
                                });

                                let tables = TableState::load_state_file();
                                let table = tables.find_table(table).unwrap_or_else(|| {
                                    panic!("ForeignKey Table '{}' not found", table)
                                });

                                if !table.is_valid_column(column) {
                                    panic!(
                                        "ForeignKey Column '{}' not found in Table '{}'",
                                        column, table.name
                                    )
                                }
                                self.alias = Some(name.to_string());

                                self.coltype =
                                    ColumnTypeDerive::ForeignKey(ColumnTypeOptionsDerive {
                                        foreign_key: name.to_string(),
                                        ..Default::default()
                                    });
                            }
                        }
                    }
                    GeekAttributeKeys::Skip | GeekAttributeKeys::Rename => {}
                }
            } else {
                // TODO(geekmasher): Handle this better
            }
        }
    }

    /// Convert the column into a list of parameters for a function
    pub(crate) fn to_params(&self) -> Option<TokenStream> {
        let identifier = &self.identifier;
        let itype = &self.itype;
        // Ignore PrimaryKey / ForeignKey / Option<T>
        if let Type::Path(TypePath { path, .. }) = itype {
            if let Some(segment) = path.segments.first() {
                match segment.ident.to_string().as_str() {
                    "Option" | "PrimaryKey" | "PrimaryKeyInteger" => return None,
                    "ForeignKey" => {
                        // We want a user to pass in the actual type in the ForeignKey
                        // so we need to extract the inner type
                        let inner_type = match segment.arguments {
                            syn::PathArguments::AngleBracketed(ref args) => {
                                args.args.first().unwrap()
                            }
                            _ => panic!("Unsupported ForeignKey type (to_params)"),
                        };
                        // Return the inner type
                        return Some(quote! {
                            #identifier: impl Into< #inner_type >
                        });
                    }
                    _ => {}
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
                match segment.ident.to_string().as_str() {
                    "Option" => {
                        // Option is always None in new()
                        return quote! {
                            #identifier: None
                        };
                    }
                    // TODO(geekmasher): Add PrimaryKey<T> support
                    "PrimaryKeyInteger" => {
                        // Generate a new primary key
                        return quote! {
                            #identifier: geekorm::PrimaryKeyInteger::new(0)
                        };
                    }
                    "ForeignKey" => {
                        // Generate a new foreign key
                        return quote! {
                            #identifier: geekorm::ForeignKey::from(#identifier.into())
                        };
                    }
                    _ => {}
                }
            }
        }
        quote! {
            #identifier
        }
    }

    pub(crate) fn get_selector(&self, table_ident: &Ident) -> TokenStream {
        let identifier = &self.identifier;
        let name = &self.name;

        let func_name = format!("select_by_{}", identifier);
        let func = Ident::new(&func_name, Span::call_site());

        quote! {
            fn #func(value: impl Into<geekorm::Value>) -> geekorm::Query {
                geekorm::QueryBuilder::select()
                    .table(#table_ident::table())
                    .where_eq(#name, value.into())
                    .build()
                    .expect("Failed to build query")
            }
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
                        unique: false,
                        not_null: false,
                        // If the inner type is an integer, auto increment
                        auto_increment: inner_type_name == "Integer",
                    })
                }
                "PrimaryKeyInteger" => ColumnTypeDerive::Integer(ColumnTypeOptionsDerive {
                    primary_key: true,
                    foreign_key: String::new(),
                    unique: false,
                    not_null: false,
                    auto_increment: true,
                }),
                "ForeignKey" => {
                    let options = ColumnTypeOptionsDerive {
                        primary_key: false,
                        foreign_key: String::from("GeekOrmForeignKey"),
                        unique: false,
                        not_null: true,
                        auto_increment: false,
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
                // TODO(geekmasher): Remove this
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
