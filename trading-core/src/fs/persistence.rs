use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;
use std::path::Path;

/// Saves a serializable object to a file atomically.
///
/// This function ensures data integrity by writing to a temporary file first
/// and then renaming it to the target path.
///
/// # Arguments
///
/// * `path` - The target file path.
/// * `state` - The object to serialize and save.
///
/// # Returns
///
/// * `Ok(())` on success.
/// * `Err` on serialization or IO failure.
pub fn save_state<T: Serialize>(path: &Path, state: &T) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create parent directory")?;
    }

    let json = serde_json::to_string_pretty(state).context("Failed to serialize state")?;

    // Create a temporary file in the same directory to ensure atomic move support
    let temp_path = path.with_extension("tmp");
    let mut temp_file = std::fs::File::create(&temp_path).context("Failed to create temp file")?;

    temp_file
        .write_all(json.as_bytes())
        .context("Failed to write to temp file")?;
    temp_file.sync_all().context("Failed to sync temp file")?; // Ensure data is on disk

    std::fs::rename(&temp_path, path).context("Failed to rename temp file to target")?;

    Ok(())
}

/// Loads a deserializable object from a file.
///
/// # Arguments
///
/// * `path` - The file path to read from.
///
/// # Returns
///
/// * `Ok(T)` containing the deserialized object.
/// * `Err` if the file doesn't exist, cannot be read, or deserialization fails.
pub fn load_state<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let file = std::fs::File::open(path).context("Failed to open state file")?;
    let reader = std::io::BufReader::new(file);
    let state = serde_json::from_reader(reader).context("Failed to deserialize state")?;
    Ok(state)
}
