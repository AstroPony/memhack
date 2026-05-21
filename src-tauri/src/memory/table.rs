use crate::memory::types::TableEntry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressTable {
    pub game_name: String,
    pub entries: Vec<TableEntry>,
}

#[allow(dead_code)]
impl AddressTable {
    pub fn new(game_name: &str) -> Self {
        Self {
            game_name: game_name.to_string(),
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: TableEntry) {
        self.entries.push(entry);
    }

    pub fn remove(&mut self, index: usize) -> Option<TableEntry> {
        if index < self.entries.len() {
            Some(self.entries.remove(index))
        } else {
            None
        }
    }
}

/// Get the directory where address tables are stored
fn tables_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_local_dir()
        .ok_or("Could not find local data directory")?
        .join("memhack")
        .join("tables");

    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create tables dir: {e}"))?;
    Ok(dir)
}

/// Sanitize a game name for use as a filename
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Save an address table to disk
pub fn save_table(table: &AddressTable) -> Result<String, String> {
    let dir = tables_dir()?;
    let filename = format!("{}.json", sanitize_name(&table.game_name));
    let path = dir.join(&filename);

    let json = serde_json::to_string_pretty(table)
        .map_err(|e| format!("Serialization error: {e}"))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

    Ok(path.display().to_string())
}

/// Load an address table from disk
pub fn load_table(game_name: &str) -> Result<AddressTable, String> {
    let dir = tables_dir()?;
    let filename = format!("{}.json", sanitize_name(game_name));
    let path = dir.join(&filename);

    let json =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))
}

/// List all saved tables
pub fn list_tables() -> Result<Vec<String>, String> {
    let dir = tables_dir()?;
    let entries = fs::read_dir(&dir).map_err(|e| format!("Failed to read tables dir: {e}"))?;

    let names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".json") {
                Some(name.trim_end_matches(".json").to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(names)
}

#[allow(dead_code)]
pub fn delete_table(game_name: &str) -> Result<(), String> {
    let dir = tables_dir()?;
    let filename = format!("{}.json", sanitize_name(game_name));
    let path = dir.join(&filename);

    fs::remove_file(&path).map_err(|e| format!("Failed to delete {}: {e}", path.display()))
}
