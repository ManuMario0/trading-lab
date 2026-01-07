//! Watchdog for available microservices.
//!
//! It provides the [`DiskWatcher`] structure that is responsible for scanning the file system
//! for microservices and emitting events when new ones are discovered, removed or modified.

use crate::event_bus::{EventBus, SystemError, SystemEvent};
use anyhow::Context;
use log::{error, info};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use orchestrator_protocol::model::{PortInfo, ServiceDescriptor};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

/// A File System Watcher that scans for Microservices.
///
/// It follows the "Side-Effect at the Edge" pattern.
/// It does the messy I/O (watching files, parsing JSON) and converts it
/// into pure `SystemEvent::ServiceDiscovered`.
///
/// # Example
///
/// ```ignore
/// let event_bus = EventBus::new();
/// let watcher = DiskWatcher::new("/path/to/services", event_bus.clone());
/// watcher.run().await;
/// ```
pub struct DiskWatcher {
    service_dir: PathBuf,
    event_bus: EventBus,
}

impl DiskWatcher {
    /// Creates a new [`DiskWatcher`].
    ///
    /// # Arguments
    ///
    /// * `service_dir` - The directory to scan for microservices.
    /// * `event_bus` - The event bus to emit events to.
    ///
    /// # Returns
    ///
    /// A new [`DiskWatcher`].
    pub fn new(service_dir: PathBuf, event_bus: EventBus) -> Self {
        Self {
            service_dir,
            event_bus,
        }
    }

    /// Run the watcher. This function blocks/loops forever, so spawn it in a task.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`DiskWatcher`] to run.
    pub async fn run(self) {
        info!("Registry: Starting Disk Watcher on {:?}", self.service_dir);

        // 1. Initial Scan
        if let Err(e) = self.scan_all().await {
            error!("Registry: Initial scan failed: {}", e);
        }

        // 2. Setup Watcher (notify crate) - Runs on a separate blocking thread usually
        let (tx, rx) = channel();

        // We configure notify with a small delay for debounce
        let config = Config::default().with_poll_interval(Duration::from_secs(2));
        let mut watcher: RecommendedWatcher = match Watcher::new(tx, config) {
            Ok(w) => w,
            Err(e) => {
                let msg = format!("Failed to create watcher: {}", e);
                error!("Registry: {}", msg);
                self.event_bus.publish(SystemEvent::Error {
                    error: SystemError::io(msg),
                });
                return;
            }
        };

        if let Err(e) = watcher.watch(&self.service_dir, RecursiveMode::NonRecursive) {
            error!("Registry: Failed to watch directory: {}", e);
            return;
        }

        // 3. Watch Loop
        let bus = self.event_bus.clone();

        tokio::task::spawn_blocking(move || {
            use std::collections::HashMap;
            use std::time::{Duration, Instant};

            // We use a HashMap to track pending updates and debounce them.
            let mut pending_updates: HashMap<PathBuf, Instant> = HashMap::new();
            // How long to wait before processing a file. A long debounce should never be a problem,
            // directory scanning is not a performance critical path.
            let debounce_duration = Duration::from_millis(1000);
            // How often to check for pending updates.
            let tick_rate = Duration::from_millis(200);

            loop {
                // We use recv_timeout to check for new events or timeout to check pending updates
                match rx.recv_timeout(tick_rate) {
                    Ok(Ok(Event { paths, kind, .. })) => {
                        // info!("Registry: File Event {:?} on {:?}", kind, paths);
                        for path in paths {
                            if path.is_file() {
                                // Reset the timer: we saw activity, so it's not settled yet.
                                pending_updates.insert(path, Instant::now());
                            }
                        }
                    }
                    Ok(Err(e)) => error!("Registry: Watch error: {}", e),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // No new events this tick, fall through to check pending list
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }

                // Check for "Settled" files (quiet for > 1s)
                let now = Instant::now();
                let settled_paths: Vec<PathBuf> = pending_updates
                    .iter()
                    .filter(|(_, last_seen)| now.duration_since(**last_seen) > debounce_duration)
                    .map(|(path, _)| path.clone())
                    .collect();

                for path in settled_paths {
                    pending_updates.remove(&path);
                    // Now we are confident the file write is likely complete
                    Self::process_file_sync(&path, &bus);
                }
            }
        });
    }

    /// Scans all files in the directory.
    ///
    /// For each file scanned, it processes it with [`process_file`].
    async fn scan_all(&self) -> anyhow::Result<()> {
        let mut dir = tokio::fs::read_dir(&self.service_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                self.process_file(&path).await;
            }
        }
        Ok(())
    }

    /// Processes a file.
    ///
    /// Each file is called witht the `manifest` parameter to query the `ServiceDescriptor`.
    async fn process_file(&self, path: &Path) {
        // We just delegate to the sync version or implement async version.
        // Since `extract_manifest` uses `std::process::Command` (blocking),
        // we should arguably wrap it or use tokio::process::Command.
        // Let's keep it simple and blocking for now (Registry scan is rare event).
        Self::process_file_sync(path, &self.event_bus);
    }

    /// Processes a file synchronously.
    ///
    /// Each file is called witht the `manifest` parameter to query the `ServiceDescriptor`.
    /// If a `ServiceDescriptor` is properly emitted by the service, it emits a [`SystemEvent::ServiceDiscovered`].
    fn process_file_sync(path: &Path, bus: &EventBus) {
        // Ignore hidden files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                return;
            }
        }

        match extract_manifest(path) {
            Ok(manifest) => {
                let descriptor = map_manifest_to_descriptor(&manifest, path);

                // CRITICAL: We emit the event. We do NOT update the `ServiceCatalog` directly.
                // The Main Loop or State Machine will listen to this event and update the catalog.
                // This ensures "Single Source of Truth" flow.
                bus.publish(SystemEvent::ServiceDiscovered {
                    descriptor: descriptor.clone(),
                });

                info!(
                    "Registry: Discovered service '{}' version '{}' from {:?}",
                    descriptor.service,
                    descriptor.version,
                    path.file_name()
                );
            }
            Err(_e) => {
                // Too noisy to log error for every non-binary file?
                // warn!("Registry: Failed to parse {:?}: {}", path, e);
            }
        }
    }
}

/// Helper: Runs `./binary manifest` to get validity and ports.
fn extract_manifest(path: &Path) -> anyhow::Result<trading_core::manifest::ServiceManifest> {
    // We expect the executable to support `manifest` subcommand.
    let output = Command::new(path)
        .arg("manifest")
        .output()
        .context("Failed to execute binary")?;

    if !output.status.success() {
        anyhow::bail!("Binary returned non-zero exit code");
    }

    let json = String::from_utf8(output.stdout)?;
    let manifest: trading_core::manifest::ServiceManifest = serde_json::from_str(&json)?;
    Ok(manifest)
}

fn map_manifest_to_descriptor(
    manifest: &trading_core::manifest::ServiceManifest,
    path: &Path,
) -> ServiceDescriptor {
    ServiceDescriptor {
        service: manifest.blueprint.service_type.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        binary_path: Some(path.to_string_lossy().to_string()),
        inputs: manifest
            .blueprint
            .inputs
            .iter()
            .map(|p| PortInfo {
                name: p.name.clone(),
                data_type: p.data_type.clone(),
                required: p.required,
                is_variadic: p.is_variadic,
            })
            .collect(),
        outputs: manifest
            .blueprint
            .outputs
            .iter()
            .map(|p| PortInfo {
                name: p.name.clone(),
                data_type: p.data_type.clone(),
                required: p.required,
                is_variadic: p.is_variadic,
            })
            .collect(),
    }
}
