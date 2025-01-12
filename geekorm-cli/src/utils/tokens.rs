use geekorm::{Column, ColumnType, ColumnTypeOptions, Columns, Table};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl ToTokens for Table {
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

impl ToTokens for Columns {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let columns = &self.columns;
        tokens.extend(quote! {
            vec![#(#columns),*]
        });
    }
}
impl ToTokens for Column {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let coltype = &self.column_type;
        let alias = &self.alias;
        let skip = &self.skip;

        tokens.extend(quote! {
            geekorm::Column {
                name: String::from(#name),
                column_type: #coltype,
                alias: String::from(#alias),
                skip: #skip,
            }
        });
    }
}

impl ToTokens for ColumnType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ColumnType::Identifier(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Identifier(#options)
                });
            }
            ColumnType::Text(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Text(#options)
                });
            }
            ColumnType::Integer(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Integer(#options)
                });
            }
            ColumnType::Boolean(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Boolean(#options)
                });
            }
            ColumnType::Blob(options) => {
                tokens.extend(quote! {
                    geekorm::ColumnType::Blob(#options)
                });
            }
            ColumnType::ForeignKey(options) => tokens.extend(quote! {
                geekorm::ColumnType::ForeignKey(#options)
            }),
        }
    }
}

impl ToTokens for ColumnTypeOptions {
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
