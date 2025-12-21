use crate::fs::{load_state, save_state, PathManager};
use crate::model::instrument::InstrumentId;
use crate::model::Instrument;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

const INSTRUMENT_DB_FILE_NAME: &str = "instruments.json";

/// A persistent database for trading instruments.
///
/// Wraps a `HashMap` of instruments and handles synchronization with a JSON file on disk.
#[derive(Debug)]
pub struct InstrumentDB {
    instruments: HashMap<InstrumentId, Instrument>,
    file_path: PathBuf,
}

impl InstrumentDB {
    /// Creates a new, empty InstrumentDB backed by the specified file path.
    ///
    /// Note: This does NOT load from the file automatically. Use `load` or `sync` to populate.
    fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            instruments: HashMap::new(),
            file_path: file_path.into(),
        }
    }

    /// Loads the database from the standard location using the PathManager.
    ///
    /// The standard filename is `instruments.json` inside the `data` directory.
    pub fn load(path_manager: &PathManager) -> Result<Self> {
        let mut db = Self::new(path_manager.get_common_file_path(INSTRUMENT_DB_FILE_NAME));

        // If the file exists, load it. If not, start empty.
        if db.file_path.exists() {
            db.sync()?;
        }

        Ok(db)
    }

    /// Reloads the database content from disk, replacing in-memory state.
    pub fn sync(&mut self) -> Result<()> {
        if !self.file_path.exists() {
            // Nothing to load, just keep current state (or clear? usually sync implies "make like disk")
            // If file missing, maybe treat as empty?
            // Better: if file missing, do nothing (assume new DB), or error?
            // "if the id is not in the db, sync the db in case it has been updated"
            // implies we expect the file to be the source of truth.
            return Ok(());
        }

        self.instruments = load_state(&self.file_path)
            .with_context(|| format!("Failed to load instruments from {:?}", self.file_path))?;
        Ok(())
    }

    /// Saves the current in-memory state to disk.
    pub fn save(&self) -> Result<()> {
        save_state(&self.file_path, &self.instruments)
            .with_context(|| format!("Failed to save instruments to {:?}", self.file_path))?;
        Ok(())
    }

    /// Retrieves an instrument by its ID.
    pub fn get(&self, id: InstrumentId) -> Option<&Instrument> {
        self.instruments.get(&id)
    }

    /// Adds or updates an instrument and persists the change to disk immediately.
    ///
    /// To perform batch updates without saving every time, access the internal map directly
    /// (not exposed yet) or add a batch method. For now, safety defaults to immediate save.
    pub fn set(&mut self, id: InstrumentId, instrument: Instrument) -> Result<()> {
        self.instruments.insert(id, instrument);
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::instrument::Stock;
    use std::env;
    use std::fs;

    #[test]
    fn test_instrument_db_persistence() -> Result<()> {
        let mut temp_path = env::temp_dir();
        // Simple unique-ish name for test to avoid collisions
        let filename = format!(
            "test_instruments_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        );
        temp_path.push(filename);

        // Ensure cleanup
        let cleanup_path = temp_path.clone();
        defer_cleanup(&cleanup_path); // Attempt cleanup on panic not guaranteed, but good enough for dev

        let mut db = InstrumentDB::new(temp_path.clone());
        let stock = Instrument::Stock(Stock::new(
            1,
            "AAPL",
            "NASDAQ",
            "Tech",
            "Consumer Electronics",
            "USA",
            "USD",
        ));

        // Set (writes to disk)
        db.set(1, stock.clone())?;

        // Verify in memory
        assert_eq!(db.get(1), Some(&stock));

        // Create a separate instance to verify load from disk
        let mut db2 = InstrumentDB::new(temp_path.clone());
        db2.sync()?;

        assert_eq!(db2.get(1), Some(&stock));

        // Cleanup
        let _ = fs::remove_file(temp_path);

        Ok(())
    }

    fn defer_cleanup(_path: &std::path::Path) {
        // In a real test utils, we might use a Drop guard, but here we just manually cleanup at end of test.
        // The helper is just to remind us.
    }
}
