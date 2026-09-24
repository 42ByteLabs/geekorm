//! # Internal CodeGen module

use geekorm_sql::{Column, ColumnOptions, ColumnType, Table};
use quote::{ToTokens, quote};

/// Database for Code Generation
pub struct CgDatabase {
    tables: Vec<CgTable>,
}

impl From<Vec<Table>> for CgDatabase {
    fn from(value: Vec<Table>) -> Self {
        Self {
            tables: value.iter().map(|t| t.clone().into()).collect(),
        }
    }
}

impl ToTokens for CgDatabase {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let tables = &self.tables;

        tokens.extend(quote! {
            geekorm::Database {
                tables: vec![
                    #(#tables ),*
                ]
            }
        });
    }
}

/// Table for Code Generation
pub struct CgTable {
    name: String,
    columns: Vec<CgColumn>,
    database: Option<String>,
}

impl ToTokens for CgTable {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let columns = &self.columns;
        let database = self.database.clone().unwrap_or("Database".to_string());

        tokens.extend(quote! {
            geekorm::Table {
                name: String::from(#name),
                columns: geekorm::Columns::from(vec![
                    #(#columns ),*
                ]),
                database: Some(String::from(#database)),
            }
        });
    }
}

impl From<Table> for CgTable {
    fn from(value: Table) -> Self {
        Self {
            name: value.name,
            columns: value.columns.iter().map(|c| c.clone().into()).collect(),
            database: value.database,
        }
    }
}

/// Column for Code Generation
pub struct CgColumn {
    name: String,
    column_type: CgColumnType,
    column_options: CgColumnOptions,
    alias: Option<String>,
    foreign_key: Option<String>,
    table_name: Option<String>,
}

impl ToTokens for CgColumn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let column_type = &self.column_type;
        let column_options = &self.column_options;
        let alias = &self.alias;
        let foreign_key = &self.foreign_key;
        let table_name = &self.table_name;

        tokens.extend(quote! {
            geekorm::Column {
                name: String::from(#name),
                column_type: #column_type,
                column_options: #column_options,
                alias: #alias,
                foreign_key: #foreign_key,
                table_name: #table_name
            }
        });
    }
}

impl From<Column> for CgColumn {
    fn from(value: Column) -> Self {
        Self {
            name: value.name,
            column_type: value.column_type.into(),
            column_options: value.column_options.into(),
            alias: value.alias,
            foreign_key: value.foreign_key,
            table_name: value.table_name,
        }
    }
}

/// ColumnOption for Code Generation
pub struct CgColumnOptions {
    primary_key: bool,
    unique: bool,
    not_null: bool,
    auto_increment: bool,
}

impl ToTokens for CgColumnOptions {
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
                auto_increment: #auto_increment
            }
        });
    }
}

impl From<ColumnOptions> for CgColumnOptions {
    fn from(value: ColumnOptions) -> Self {
        Self {
            primary_key: value.primary_key,
            unique: value.unique,
            not_null: value.not_null,
            auto_increment: value.auto_increment,
        }
    }
}

/// ColumnType for Code Generation
pub enum CgColumnType {
    /// Text type
    Text,
    /// Int type
    Integer,
    /// Bool type
    Boolean,
    /// Byte array type
    Blob,
    /// Foreign key type
    ForeignKey,
}

impl ToTokens for CgColumnType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.to_string();
        tokens.extend(quote! {
            geekorm::ColumnType::#ident
        });
    }
}

impl ToString for CgColumnType {
    fn to_string(&self) -> String {
        match self {
            CgColumnType::Text => String::from("Text"),
            CgColumnType::Integer => String::from("Integer"),
            CgColumnType::Boolean => String::from("Boolean"),
            CgColumnType::Blob => String::from("Blob"),
            CgColumnType::ForeignKey => String::from("ForeignKey"),
        }
    }
}

impl From<ColumnType> for CgColumnType {
    fn from(value: ColumnType) -> Self {
        match value {
            ColumnType::Text => Self::Text,
            ColumnType::Integer => Self::Integer,
            ColumnType::Boolean => Self::Boolean,
            ColumnType::Blob => Self::Blob,
            ColumnType::ForeignKey => Self::ForeignKey,
        }
    }
}
