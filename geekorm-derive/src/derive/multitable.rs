//! MultiTable module

use geekorm_core::{Column, ColumnType, ColumnTypeOptions, Table};

use crate::internal::TableState;

use super::{ColumnDerive, ColumnTypeDerive, ColumnTypeOptionsDerive, ColumnsDerive, TableDerive};

/// Create a One-to-Many relationship between two tables by creating a new table
/// with the primary key of the first and second tables are foreign keys in the third table.
///
/// ```rust
/// use geekorm::prelude::*;
/// use geekorm::PrimaryKey;
///
/// #[derive(GeekTable)]
/// struct Users {
///     id: PrimaryKey<i32>,
///     username: String,
///
///     #[geekorm(foreign_key = "Sessions.id")]
///     sessions: Vec<Sessions>,
/// }
///
/// #[derive(GeekTable)]
/// struct Sessions {
///     id: PrimaryKey<i32>,
///     #[geekorm(rand)]
///     token: String
/// }
///
/// ```
pub(crate) fn one_to_many(
    table: &TableDerive,
    column: &ColumnDerive,
    foreign_table: &str,
) -> Result<Table, syn::Error> {
    let table_name = format!("{}_{}", table.name, foreign_table);

    let mut new_table = Table {
        name: table_name,
        columns: Default::default(),
    };

    new_table.columns.columns.push(Column {
        name: String::from("id"),
        column_type: ColumnType::Identifier(ColumnTypeOptions {
            primary_key: true,
            unique: true,
            not_null: true,
            auto_increment: true,
            foreign_key: String::new(),
        }),
        ..Default::default()
    });

    let table_pk = match table.get_primary_key() {
        Some(pk) => pk,
        None => {
            return Err(syn::Error::new_spanned(
                table,
                format!(
                    "Table `{}` must have a primary key to create a one-to-many relationship",
                    table.name
                ),
            ))
        }
    };

    new_table.columns.columns.push(Column {
        name: format!("{}_id", table.name.to_lowercase()),
        column_type: ColumnType::ForeignKey(ColumnTypeOptions {
            foreign_key: format!("{}.{}", table.name, table_pk.name),
            not_null: true,
            ..Default::default()
        }),
        ..Default::default()
    });

    let foreign_table_key = match column.coltype {
        ColumnTypeDerive::ForeignKey(ref key) => key.foreign_key.clone(),
        _ => {
            return Err(syn::Error::new_spanned(
                column,
                "Column must be a foreign key to create a one-to-many relationship",
            ))
        }
    };

    new_table.columns.columns.push(Column {
        name: format!("{}_id", foreign_table.to_lowercase()),
        column_type: ColumnType::ForeignKey(ColumnTypeOptions {
            foreign_key: foreign_table_key,
            not_null: true,
            ..Default::default()
        }),
        ..Default::default()
    });

    TableState::add(new_table.clone());

    Ok(new_table)
}
