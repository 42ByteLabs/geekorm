use crate::ToSqlite;

#[derive(Debug, Clone)]
pub enum ColumnType {
    Text(ColumnTypeOptions),
    Integer(ColumnTypeOptions),
}

impl ToSqlite for ColumnType {
    fn on_create(&self) -> String {
        match self {
            ColumnType::Text(options) => {
                let opts = options.on_create();
                if opts.is_empty() {
                    return "TEXT".to_string();
                }
                format!("TEXT {}", options.on_create())
            }
            ColumnType::Integer(options) => {
                let opts = options.on_create();
                if opts.is_empty() {
                    return "INTEGER".to_string();
                }
                format!("INTEGER {}", options.on_create())
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ColumnTypeOptions {
    pub primary_key: bool,
    pub unique: bool,
    pub not_null: bool,
}

impl ColumnTypeOptions {
    pub(crate) fn primary_key() -> Self {
        ColumnTypeOptions {
            primary_key: true,
            not_null: true,
            ..Default::default()
        }
    }

    pub(crate) fn unique() -> Self {
        ColumnTypeOptions {
            unique: true,
            ..Default::default()
        }
    }

    pub(crate) fn null() -> Self {
        ColumnTypeOptions {
            not_null: false,
            ..Default::default()
        }
    }
}

impl ToSqlite for ColumnTypeOptions {
    fn on_create(&self) -> String {
        let mut sql = Vec::new();
        if self.not_null {
            sql.push("NOT NULL");
        }
        if self.primary_key {
            sql.push("PRIMARY KEY");
        }
        if self.unique {
            sql.push("UNIQUE");
        }
        sql.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_type_to_sql() {
        let column_type = ColumnType::Text(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(), "TEXT");

        let column_type = ColumnType::Integer(ColumnTypeOptions::default());
        assert_eq!(column_type.on_create(), "INTEGER");
    }

    #[test]
    fn test_column_type_options_to_sql() {
        let column_type_options = ColumnTypeOptions::default();
        assert_eq!(column_type_options.on_create(), "");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.on_create(), "PRIMARY KEY");

        let column_type_options = ColumnTypeOptions {
            primary_key: true,
            not_null: true,
            ..Default::default()
        };
        assert_eq!(column_type_options.on_create(), "NOT NULL PRIMARY KEY");
    }
}
