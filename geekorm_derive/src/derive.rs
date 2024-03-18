use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use geekorm::Table;
use syn::{Type, TypePath};

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

#[derive(Debug, Clone)]
pub(crate) struct ColumnsDerive {
    pub(crate) columns: Vec<ColumnDerive>,
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

impl From<Vec<ColumnDerive>> for ColumnsDerive {
    fn from(columns: Vec<ColumnDerive>) -> Self {
        ColumnsDerive { columns }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ColumnDerive {
    pub(crate) name: String,
    pub(crate) coltype: ColumnTypeDerive,
}

impl ColumnDerive {
    pub(crate) fn new(name: String, coltype: ColumnTypeDerive) -> Self {
        ColumnDerive { name, coltype }
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

#[derive(Debug, Clone)]
pub(crate) enum ColumnTypeDerive {
    Text(ColumnTypeOptionsDerive),
    Integer(ColumnTypeOptionsDerive),
    Boolean(ColumnTypeOptionsDerive),
}

impl ToTokens for ColumnTypeDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
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
        }
    }
}

impl From<&Type> for ColumnTypeDerive {
    fn from(ty: &Type) -> Self {
        match ty {
            Type::Slice(_) => ColumnTypeDerive::Text(ColumnTypeOptionsDerive::default()),
            Type::Path(path) => {
                if path.path.is_ident("String") {
                    ColumnTypeDerive::Text(ColumnTypeOptionsDerive::default())
                } else if path.path.is_ident("i32")
                    || path.path.is_ident("i64")
                    || path.path.is_ident("u32")
                    || path.path.is_ident("u64")
                {
                    ColumnTypeDerive::Integer(ColumnTypeOptionsDerive::default())
                } else if path.path.is_ident("bool") {
                    ColumnTypeDerive::Boolean(ColumnTypeOptionsDerive::default())
                } else {
                    panic!("Unsupported column type :: {:?}", ty.to_token_stream())
                }
            }
            _ => panic!("Unsupported column type :: {:?}", ty.to_token_stream()),
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub(crate) struct ColumnTypeOptionsDerive {
    pub(crate) primary_key: bool,
    pub(crate) unique: bool,
    pub(crate) not_null: bool,
}

impl ToTokens for ColumnTypeOptionsDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let primary_key = &self.primary_key;
        let unique = &self.unique;
        let not_null = &self.not_null;

        tokens.extend(quote! {
            geekorm::ColumnTypeOptions {
                primary_key: #primary_key,
                unique: #unique,
                not_null: #not_null
            }
        });
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
