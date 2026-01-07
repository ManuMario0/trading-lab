//! Builder for fusing user strategies with the trading engine.
//!
//! This module handles the "Fusion" phase of the Forge workflow.
//! It takes a user-defined strategy library, creates a temporary "Chassis" application,
//! configures it to link against the user's library, and compiles it into a standalone executable.

use crate::error::{ForgeError, Result};
use fs_extra::dir::{copy, CopyOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::{Document, Item, Value}; // Using valid Document import

/// Fuses a user strategy project into a runnable engine binary.
///
/// This process involves:
/// 1. Verifying the user's project structure.
/// 2. Creating a temporary build environment.
/// 3. Injecting the user's crate as a dependency into the Chassis template.
/// 4. Compiling the Chassis with the user's code statically linked.
/// 5. Returning the path to the compiled artifact.
///
/// # Arguments
///
/// * `user_project_path` - Path to the user's library crate (containing `Cargo.toml`).
///
/// # Returns
///
/// * `Result<PathBuf>` - The absolute path to the compiled binary artifact.
pub fn fuse_strategy(user_project_path: &Path) -> Result<PathBuf> {
    // 1. Verify user project
    let user_cargo_path = user_project_path.join("Cargo.toml");
    if !user_cargo_path.exists() {
        return Err(ForgeError::MissingCargoToml(
            user_project_path.to_path_buf(),
        ));
    }

    // Get absolute path for correctness
    let user_project_abs = fs::canonicalize(user_project_path)?;

    // 2. Create temp workspace
    // We use a tempdir that persists until function returns?
    // Actually we need it to exist during build.
    let temp_dir = tempfile::tempdir()?;
    let build_root = temp_dir.path().to_path_buf();

    // 3. Copy Chassis template
    let template_path = PathBuf::from("forge/src/internal/templates/chassis_app");
    if !template_path.exists() {
        return Err(ForgeError::TemplateNotFound(template_path));
    }

    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = true;

    // Copy Chassis into build_root
    copy(&template_path, &build_root, &options)?;
    // The chassis is now at build_root/chassis_app
    let chassis_path = build_root.join("chassis_app");

    // 4. Inject User Dependency
    inject_dependency(&chassis_path, &user_project_abs)?;

    // 5. Build
    log::info!("Compiling fused engine...");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&chassis_path)
        .status()?;

    if !status.success() {
        // We probably want to capture stderr to give better error messages
        return Err(ForgeError::CargoBuild("Compilation failed".to_string()));
    }

    // 6. Locate Artifact
    // default target dir is chassis_path/target/release/chassis
    let artifact = chassis_path.join("target/release/chassis");

    if !artifact.exists() {
        return Err(ForgeError::ArtifactNotFound(artifact));
    }

    // 7. Extract Artifact (Copy to user's target dir?)
    // For now we just return the path in temp dir.
    // The caller (main.rs) should probably move it to a persistent location.

    // We must NOT drop temp_dir yet if we return a path to it.
    // Actually `temp_dir` will drop at end of scope and delete files.
    // So we MUST move the artifact out before returning.

    let output_name = format!(
        "{}-engine",
        user_project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    let user_target_dir = user_project_path.join("target");
    fs::create_dir_all(&user_target_dir)?;

    let final_artifact = user_target_dir.join(&output_name);
    fs::copy(&artifact, &final_artifact)?;

    Ok(final_artifact)
}

/// Modifies the Chassis Cargo.toml to point `user_crate` to the user's path.
fn inject_dependency(chassis_path: &Path, user_project_path: &Path) -> Result<()> {
    // 1. Read User's Metadata
    let user_toml_path = user_project_path.join("Cargo.toml");
    let user_toml_content = fs::read_to_string(&user_toml_path)?;
    let user_doc = user_toml_content.parse::<Document>()?;

    let user_package = &user_doc["package"];
    let user_package_name = user_package["name"]
        .as_str()
        .ok_or_else(|| ForgeError::CargoBuild("User Cargo.toml missing name".to_string()))?;

    // Optional metadata (defaults if missing)
    let user_version = user_package["version"].as_str().unwrap_or("0.1.0");
    let user_description = user_package["description"].as_str().unwrap_or("");

    // 2. Modify Chassis Cargo.toml
    let toml_path = chassis_path.join("Cargo.toml");
    let content = fs::read_to_string(&toml_path)?;
    let mut doc = content.parse::<Document>()?;

    // INHERIT METADATA
    doc["package"]["version"] = Item::Value(Value::from(user_version));
    doc["package"]["description"] = Item::Value(Value::from(user_description));

    // Fix dependency paths
    let user_path_str = user_project_path.to_string_lossy().to_string();
    let workspace_root = std::env::current_dir()?;

    // We construct the inline table: { path = "ABS_PATH", package = "REAL_NAME" }
    let mut table = toml_edit::InlineTable::default();
    table.insert("path", Value::from(user_path_str));
    table.insert("package", Value::from(user_package_name));

    // We bind it to `user_crate` alias so our main.rs code `use user_crate::entry_point` works!
    doc["dependencies"]["user_crate"] = Item::Value(Value::InlineTable(table));

    // Also we need to fix the `trading` and `trading-core` paths in Chassis
    // because they are relative `../../` in the template, but in /tmp/ they won't work.
    // We should point them to the absolute path of the current workspace.
    // Assumption: We are running forge from the workspace root.
    let trading_api = workspace_root
        .join("trading-api")
        .to_string_lossy()
        .to_string();
    let trading_core = workspace_root
        .join("trading-core")
        .to_string_lossy()
        .to_string();

    let mut api_table = toml_edit::InlineTable::default();
    api_table.insert("path", Value::from(trading_api));
    doc["dependencies"]["trading"] = Item::Value(Value::InlineTable(api_table));

    let mut core_table = toml_edit::InlineTable::default();
    core_table.insert("path", Value::from(trading_core));
    doc["dependencies"]["trading-core"] = Item::Value(Value::InlineTable(core_table));

    fs::write(toml_path, doc.to_string())?;
    Ok(())
}
