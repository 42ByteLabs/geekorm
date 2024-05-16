/// This is an internal module for storing the state of the database / tables / columns.
/// How this works is that it stores the state of the database in a JSON file in
/// the build directory.
///
/// In the future we might allow the user to specify the state file location and
/// store the state in their project directory.
use std::path::PathBuf;

use geekorm_core::Table;
use serde::{Deserialize, Serialize};

/// The directory where the state is stored
/// to keep track of the database, tables, and columns created
const GEEKORM_STATE_FILE: &str = env!("GEEKORM_STATE_FILE");

/// The Table / Database state tracking macro
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TableState {
    /// The time the state was created
    pub(crate) created_at: chrono::DateTime<chrono::Utc>,
    /// The time the state was updated last
    pub(crate) updated_at: chrono::DateTime<chrono::Utc>,

    pub(crate) tables: Vec<Table>,
}

impl TableState {
    pub(crate) fn new() -> Self {
        let table = Self {
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tables: Vec::new(),
        };
        Self::write(&table);
        table
    }

    pub(crate) fn get_state_file() -> PathBuf {
        PathBuf::from(GEEKORM_STATE_FILE)
    }

    pub(crate) fn load_state_file() -> Self {
        let state_file = Self::get_state_file();
        // If the state file does not exist, create a new state file
        if !state_file.exists() {
            return Self::new();
        }
        let state_json = std::fs::read_to_string(state_file)
            .expect("[geekorm-internal] Failed to read state file");
        // Some times the state file might be empty
        match serde_json::from_str(&state_json) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("[geekorm-internal] Failed to deserialize state file: {}", e);
                TableState::new()
            }
        }
    }

    pub(crate) fn write(state: &Self) {
        let state_file = Self::get_state_file();
        let state_json = serde_json::to_string(state)
            .expect("[geekorm-internal] Failed to serialize state (write)");
        std::fs::write(state_file, state_json)
            .expect("[geekorm-internal] Failed to write state file");
    }

    pub(crate) fn add(table: Table) {
        let mut state = Self::load_state_file();
        // Remove the table if it already exists
        state.tables.retain(|t| t.name != table.name);
        state.tables.push(table);

        state.updated_at = chrono::Utc::now();

        Self::write(&state);
    }

    // Helper functions
    #[allow(dead_code)]
    pub(crate) fn find_table(&self, name: &str) -> Option<Table> {
        self.tables.iter().find(|table| table.name == name).cloned()
    }
}
