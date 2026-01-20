use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use toml::Table;

/// Check SDK dependency compatibility for a Rust project.
///
/// Responsibilities:
/// Read Cargo.toml in the project root, parse the `vtx-sdk` dependency version,
/// and compare it against the SDK metadata version bundled with the CLI.
///
/// Behavior:
/// - If Cargo.toml is missing, skip silently (not a Rust project).
/// - If versions are incompatible:
///   - By default, return an error and stop the build.
///   - If `force` is true, print a warning only.
pub fn check_rust_sdk_version(project_dir: &Path, force: bool) -> Result<()> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;
    let table: Table = toml::from_str(&content)?;

    // Get vtx-sdk version from dependencies, then dev-dependencies.
    let version = table
        .get("dependencies")
        .and_then(|d| d.get("vtx-sdk"))
        .or_else(|| table.get("dev-dependencies").and_then(|d| d.get("vtx-sdk")));

    match version {
        Some(v) => {
            let user_ver = v
                .as_str()
                .or_else(|| v.get("version").and_then(|value| value.as_str()))
                .unwrap_or("unknown");
            let cli_target_ver = vtx_sdk::VERSION; // From SDK constant.

            // Check version compatibility.
            if !is_compatible(user_ver, cli_target_ver) {
                let msg = format!(
                    "SDK Version Mismatch: Plugin uses vtx-sdk {user_ver}, but this CLI is optimized for v{cli_target_ver}."
                );

                if force {
                    println!("{} {} (Force build enabled)", "[WARN]".yellow(), msg);
                } else {
                    anyhow::bail!(
                        "{msg}\nHint: Update vtx-sdk in Cargo.toml or use --force to bypass."
                    );
                }
            } else {
                println!(
                    "{} SDK compatibility check passed (v{})",
                    "[INFO]".cyan(),
                    user_ver
                );
            }
        }
        None => {
            // Rust project without vtx-sdk might be raw Wasm or indirect deps.
            println!(
                "{} Warning: 'vtx-sdk' dependency not found in Cargo.toml.",
                "[WARN]".yellow()
            );
        }
    }
    Ok(())
}

/// Read the declared vtx-sdk version from Cargo.toml in a Rust project.
pub fn read_rust_sdk_version(project_dir: &Path) -> Option<String> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cargo_toml_path).ok()?;
    let table: Table = toml::from_str(&content).ok()?;

    let version = table
        .get("dependencies")
        .and_then(|d| d.get("vtx-sdk"))
        .or_else(|| table.get("dev-dependencies").and_then(|d| d.get("vtx-sdk")))?;

    let user_ver = version
        .as_str()
        .or_else(|| version.get("version").and_then(|value| value.as_str()))?;

    Some(user_ver.trim_start_matches(['^', '~', '=']).to_string())
}

/// Simple version compatibility check.
///
/// Logic:
/// Remove semver prefixes (^, ~, =) and require an exact match.
fn is_compatible(user: &str, system: &str) -> bool {
    let clean_user = user.trim_start_matches(['^', '~', '=']);
    clean_user == system
}
