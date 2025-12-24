use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::{Deserialize, Serialize};

/// Manages standard directory paths for the application
pub struct PathManager {
    config_dir: PathBuf,
    common_dir: PathBuf,
    temp_dir: PathBuf,
}

impl PathManager {
    /// Creates a new PathManager with explicit paths
    fn new(config_dir: impl Into<PathBuf>, common_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
            common_dir: common_dir.into(),
            temp_dir: env::temp_dir().join("trading_core"),
        }
    }

    /// Creates a PathManager from the common arguments.
    ///
    /// Sets up `config`, `data` (common), and `temp` directories.
    ///
    /// # Arguments
    ///
    /// * `args` - Parsed CLI arguments.
    ///
    /// # Returns
    ///
    /// A new `PathManager`.
    pub fn from_args(args: &crate::args::CommonArgs) -> Self {
        Self::new(args.get_config_dir(), args.get_data_dir())
    }

    /// Saves a configuration object to the `config` directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path inside config dir (e.g. "params.json").
    /// * `config` - Serializable object.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success.
    /// * `Err` on IO error.
    pub fn save_config<T>(&self, path: &Path, config: T) -> std::io::Result<()>
    where
        T: Serialize,
    {
        let config_path = self.config_dir.join(path);
        fs::write(config_path, serde_json::to_string(&config)?)
    }

    /// Loads a configuration object from the `config` directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path inside config dir.
    ///
    /// # Returns
    ///
    /// * `Ok(T)` on success.
    /// * `Err` on IO or parse error.
    pub fn load_config<T>(&self, path: &Path) -> std::io::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config_path = self.config_dir.join(path);
        let file = fs::File::open(config_path)?;
        let config = std::io::BufReader::new(file);
        let config = serde_json::from_reader(config)?;
        Ok(config)
    }

    pub fn save_common<T>(&self, path: &Path, data: T) -> std::io::Result<()>
    where
        T: Serialize,
    {
        let data_path = self.common_dir.join(path);
        fs::write(data_path, serde_json::to_string(&data)?)
    }

    pub fn load_common<T>(&self, path: &Path) -> std::io::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data_path = self.common_dir.join(path);
        let file = fs::File::open(data_path)?;
        let data = std::io::BufReader::new(file);
        let data = serde_json::from_reader(data)?;
        Ok(data)
    }

    pub fn save_temp<T>(&self, path: &Path, data: T) -> std::io::Result<()>
    where
        T: Serialize,
    {
        let temp_path = self.temp_dir.join(path);
        fs::write(temp_path, serde_json::to_string(&data)?)
    }

    pub fn load_temp<T>(&self, path: &Path) -> std::io::Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let temp_path = self.temp_dir.join(path);
        let file = fs::File::open(temp_path)?;
        let data = std::io::BufReader::new(file);
        let data = serde_json::from_reader(data)?;
        Ok(data)
    }

    /// Ensures all managed directories exist, creating them if necessary.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if directories exist or were created.
    /// * `Err` if creation fails.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.common_dir)?;
        std::fs::create_dir_all(&self.temp_dir)?;
        Ok(())
    }

    pub fn get_common_file_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.common_dir.join(path)
    }
}
