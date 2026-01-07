//! Generators for creating new trading components (Strategies, Multiplexers).
//!
//! This module handles the "Scaffolding" phase of the Forge workflow.
//! It clones internal templates to user-specified paths and customizes them
//! (e.g., renaming structs, updating Cargo.toml).

use crate::error::{ForgeError, Result};
use fs_extra::dir::{copy, CopyOptions};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{Document, Item, Value};

/// Generates a new user project from the skeleton template.
///
/// This function:
/// 1. Locates the internal `skeleton_lib` template.
/// 2. Copies it to the specified `output_path`.
/// 3. Renames the folder to match `name`.
/// 4. Updates `Cargo.toml` with the new project name.
/// 5. Updates `lib.rs` with a PascalCase struct name.
///
/// # Arguments
///
/// * `name` - The name of the new project (kebab-case recommended, e.g., "my-strategy").
/// * `output_path` - The parent directory where the project will be created.
///
/// # Returns
///
/// * `Result<()>` - Ok if successful, or a `ForgeError`.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// // forge::generator::generate_strategy("my-super-bot", Path::new("./bots"))?;
/// ```
pub fn generate_strategy(name: &str, output_path: &Path) -> Result<()> {
    // 1. Resolve internal template path
    // TODO: Improve path resolution for installed binaries
    let template_path = PathBuf::from("forge/src/internal/templates/skeleton_lib");

    if !template_path.exists() {
        return Err(ForgeError::TemplateNotFound(template_path));
    }

    // 2. Copy directory
    let mut options = CopyOptions::new();
    options.copy_inside = true;
    options.overwrite = false;

    // Create parent dir if needed
    if !output_path.exists() {
        fs::create_dir_all(output_path)?;
    }

    // Target directory
    let target_dir = output_path.join(name);
    if target_dir.exists() {
        return Err(ForgeError::DirectoryExists(target_dir));
    }

    // Copy content
    copy(&template_path, output_path, &options)?;

    // fs_extra copies the FOLDER "skeleton_lib", so we have output_path/skeleton_lib.
    let copied_dir = output_path.join("skeleton_lib");
    fs::rename(&copied_dir, &target_dir).map_err(|e| ForgeError::RenameError {
        from: copied_dir,
        to: target_dir.clone(),
        source: e,
    })?;

    // 3. Customize Cargo.toml
    customize_cargo_toml(&target_dir, name)?;

    // 4. Customize lib.rs (Struct Name)
    customize_lib_rs(&target_dir, name)?;

    println!("Forged new strategy '{}' at {}", name, target_dir.display());
    Ok(())
}

/// Updates the `Cargo.toml` of the generated project.
///
/// Sets `package.name` to the provided name.
///
/// # Arguments
///
/// * `project_dir` - The root of the new project module.
/// * `name` - The new package name.
fn customize_cargo_toml(project_dir: &Path, name: &str) -> Result<()> {
    let toml_path = project_dir.join("Cargo.toml");
    let toml_content = fs::read_to_string(&toml_path)?;
    let mut doc = toml_content.parse::<Document>()?;

    doc["package"]["name"] = Item::Value(Value::from(name));

    fs::write(toml_path, doc.to_string())?;
    Ok(())
}

/// Updates the `lib.rs` to match the project name.
///
/// Replaces the placeholder `MyStrategy` with a PascalCase version of the project name.
/// e.g. "my-bot" -> "MyBot".
///
/// # Arguments
///
/// * `project_dir` - The root of the new project module.
/// * `name` - The kebab-case project name.
fn customize_lib_rs(project_dir: &Path, name: &str) -> Result<()> {
    let struct_name = to_pascal_case(name);
    let lib_path = project_dir.join("src/lib.rs");
    let content = fs::read_to_string(&lib_path)?;

    let new_content = content.replace("MyStrategy", &struct_name);

    fs::write(lib_path, new_content)?;
    Ok(())
}

/// Converts an arbitrary string to PascalCase used for Struct names.
///
/// Use strict filtering to ensure the result is a valid Rust identifier (mostly).
/// Only alphanumeric characters are kept. Everything else acts as a separator.
///
/// # Examples
///
/// * `my-strategy` -> `MyStrategy`
/// * `super_bot` -> `SuperBot`
/// * `cool strat` -> `CoolStrat`
/// * `123test` -> `Test` (Leading numbers stripped if they would make invalid ident? Structs can't start with numbers)
///
/// Note: This function ensures the result is Alphanumeric, but doesn't strictly guarantee valid Rust ident (e.g. if input is "123").
/// For the purpose of this tool, we assume 'name' is somewhat reasonable, but we clean it up.
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_alphanumeric() {
            if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        } else {
            // Any non-alphanumeric char is treated as a separator
            capitalize_next = true;
        }
    }

    // Safety check: Empty string fallback
    if result.is_empty() {
        return "MyStrategy".to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_pascal_case_basic() {
        assert_eq!(to_pascal_case("my-strategy"), "MyStrategy");
        assert_eq!(to_pascal_case("super_bot"), "SuperBot");
        assert_eq!(to_pascal_case("cool strat"), "CoolStrat");
        assert_eq!(to_pascal_case("alreadyPascal"), "AlreadyPascal");
    }

    #[test]
    fn test_pascal_case_weird_chars() {
        assert_eq!(to_pascal_case("my@strategy#cool"), "MyStrategyCool");
        assert_eq!(to_pascal_case("___weird___"), "Weird");
    }

    #[test]
    fn test_fuzz_sanitization() {
        // Run 1000 random strings
        let mut rng = rand::rng();

        for _ in 0..1000 {
            // Generate random ascii string of length 1..50
            let len = rng.random_range(1..50);
            let s: String = (0..len)
                .map(|_| {
                    // Generate completely random u8 char to confirm filtering works
                    rng.random::<u8>() as char
                })
                .collect();

            let result = to_pascal_case(&s);

            // Assertion: It must contain ONLY alphanumeric characters or be empty (if fallback triggers)
            // Our fallback is "MyStrategy", so it's always alphanumeric.
            for c in result.chars() {
                assert!(
                    c.is_alphanumeric(),
                    "Failed fuzz test for input: {:?} -> output: {:?} contains non-alphanumeric",
                    s,
                    result
                );
            }
        }
    }
}
