//! # Testing

use crate::{Column, ColumnOptions, ColumnType, Columns, Table};

/// Testing Users table
pub fn table_users() -> Table {
    Table::new(
        "Users",
        Columns::new(vec![
            Column::from((
                "id".to_string(),
                ColumnType::Integer,
                ColumnOptions::primary_key(),
            )),
            Column::from(("username".to_string(), ColumnType::Text)),
            Column::from((
                "email".to_string(),
                ColumnType::Text,
                ColumnOptions::unique(),
            )),
            Column::new_foreign_key("roles", "Roles.id"),
            Column::new_foreign_key("profile", "Images.id"),
        ]),
    )
}

pub fn table_roles() -> Table {
    Table::new(
        "Roles",
        Columns::new(vec![Column::from((
            "id".to_string(),
            ColumnType::Integer,
            ColumnOptions::primary_key(),
        ))]),
    )
}

pub fn table_images() -> Table {
    Table::new(
        "Images",
        Columns::new(vec![
            Column::from((
                "id".to_string(),
                ColumnType::Integer,
                ColumnOptions::primary_key(),
            )),
            Column::from(("title".to_string(), ColumnType::Text)),
            Column::from(("url".to_string(), ColumnType::Text)),
        ]),
    )
}
