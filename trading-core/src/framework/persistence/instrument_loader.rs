use crate::fs::{load_state, save_state, PathManager};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use trading::model::instrument::{Instrument, InstrumentId};
use trading::model::instrument_db::InstrumentDB;

const INSTRUMENT_DB_FILE_NAME: &str = "instruments.json";

/// Helper to load InstrumentDB from disk.
pub fn load_instrument_db(path_manager: &PathManager) -> Result<InstrumentDB> {
    let file_path = path_manager.get_common_file_path(INSTRUMENT_DB_FILE_NAME);
    let mut db = InstrumentDB::new();

    if !file_path.exists() {
        return Ok(db);
    }

    let instruments: HashMap<InstrumentId, Instrument> = load_state(&file_path)
        .with_context(|| format!("Failed to load instruments from {:?}", file_path))?;

    // We need to insert them into the DB one by one since internal fields might be private
    // Wait, InstrumentDB fields are private?
    // instruments field in InstrumentDB IS private.
    // And I only exposed `insert`.

    for (id, instrument) in instruments {
        db.insert(id, instrument);
    }

    Ok(db)
}

/// Helper to save InstrumentDB to disk.
pub fn save_instrument_db(db: &InstrumentDB, path_manager: &PathManager) -> Result<()> {
    let file_path = path_manager.get_common_file_path(INSTRUMENT_DB_FILE_NAME);

    // Convert DB items to HashMap for saving
    let instruments: HashMap<InstrumentId, Instrument> =
        db.iter().map(|(k, v)| (*k, v.clone())).collect();

    save_state(&file_path, &instruments)
        .with_context(|| format!("Failed to save instruments to {:?}", file_path))?;
    Ok(())
}
