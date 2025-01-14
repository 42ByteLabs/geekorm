//! # Alter

/// Alter mode
#[derive(Debug)]
pub enum AlterMode {
    /// Add a table
    AddTable,
    /// Rename a table
    RenameTable,
    /// Drop a table
    DropTable,

    /// Add a column
    AddColumn,
    /// Rename a column
    RenameColumn,
    /// Drop a column
    DropColumn,

    /// Skip
    Skip,
}

/// Alter query builder
#[derive(Debug)]
pub struct AlterQuery {
    mode: AlterMode,
    table: String,
    column: String,

    rename: Option<String>,
}

impl AlterQuery {
    /// Create a new alter query
    pub fn new(mode: AlterMode, table: impl Into<String>, column: impl Into<String>) -> Self {
        Self {
            mode,
            table: table.into(),
            column: column.into(),
            rename: None,
        }
    }

    /// Rename the table
    pub fn rename(&mut self, name: impl Into<String>) -> &mut Self {
        self.rename = Some(name.into());
        self
    }

    /// Build the query
    /// https://sqlite.org/lang_altertable.html
    pub fn build(&self) -> String {
        match self.mode {
            AlterMode::AddTable => {
                format!("ALTER TABLE {} ADD COLUMN {};", self.table, self.column)
            }
            AlterMode::RenameTable => {
                format!("ALTER TABLE {} RENAME TO {};", self.table, self.column)
            }
            AlterMode::DropTable => {
                format!("ALTER TABLE {} DROP COLUMN {};", self.table, self.column)
            }
            AlterMode::AddColumn => {
                format!("ALTER TABLE {} ADD COLUMN {};", self.table, self.column)
            }
            AlterMode::RenameColumn => {
                format!(
                    "ALTER TABLE {} RENAME COLUMN {} TO {};",
                    self.table,
                    self.column,
                    self.rename.as_ref().unwrap_or(&self.column)
                )
            }
            AlterMode::DropColumn => {
                format!("ALTER TABLE {} DROP COLUMN {};", self.table, self.column)
            }
            AlterMode::Skip => {
                if self.column.is_empty() {
                    format!("-- Skipping {} this migration", self.table)
                } else {
                    format!("-- Skipping {}.{} this migration", self.table, self.column)
                }
            }
        }
    }
}
