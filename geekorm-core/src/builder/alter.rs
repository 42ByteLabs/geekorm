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
    pub(crate) mode: AlterMode,

    /// Table name
    pub(crate) table: String,
    /// Column name
    pub(crate) column: String,
    /// Rename column (if applicable)
    pub(crate) rename: Option<String>,
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
}
