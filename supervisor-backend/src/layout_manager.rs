use anyhow::Result;
use log::{error, info};
use orchestrator_protocol::Layout;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::RwLock;

const LAYOUTS_FILE: &str = "layouts.json";

#[derive(Serialize, Deserialize)]
struct LayoutStore {
    layouts: HashMap<String, Layout>,
}

pub struct LayoutManager {
    store: RwLock<LayoutStore>,
}

impl LayoutManager {
    pub fn new() -> Self {
        let store = if Path::new(LAYOUTS_FILE).exists() {
            info!("Loading layouts from {}", LAYOUTS_FILE);
            match fs::read_to_string(LAYOUTS_FILE) {
                Ok(c) => serde_json::from_str(&c).unwrap_or_else(|e| {
                    error!("Failed to parse layouts: {}", e);
                    LayoutStore {
                        layouts: HashMap::new(),
                    }
                }),
                Err(e) => {
                    error!("Failed to read layouts: {}", e);
                    LayoutStore {
                        layouts: HashMap::new(),
                    }
                }
            }
        } else {
            info!("No layouts file found. Starting empty.");
            LayoutStore {
                layouts: HashMap::new(),
            }
        };

        Self {
            store: RwLock::new(store),
        }
    }

    pub fn get_layout(&self, id: &str) -> Option<Layout> {
        let read = self.store.read().unwrap();
        read.layouts.get(id).cloned()
    }

    pub fn save_layout(&self, id: String, layout: Layout) -> Result<()> {
        {
            let mut write = self.store.write().unwrap();
            write.layouts.insert(id, layout);
        } // Drop lock before saving to disk
        self.persist()
    }

    pub fn remove_layout(&self, id: &str) -> Result<()> {
        {
            let mut write = self.store.write().unwrap();
            write.layouts.remove(id);
        }
        self.persist()
    }

    fn persist(&self) -> Result<()> {
        let read = self.store.read().unwrap();
        let json = serde_json::to_string_pretty(&*read)?;
        fs::write(LAYOUTS_FILE, json)?;
        Ok(())
    }
}
