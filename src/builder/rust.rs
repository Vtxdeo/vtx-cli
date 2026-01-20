use super::Builder;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rust builder.
///
/// Responsibilities: wrap Cargo toolchain calls to build Rust plugins.
pub struct RustBuilder;

impl Builder for RustBuilder {
    /// Check cargo toolchain availability.
    fn check_env(&self) -> Result<()> {
        Command::new("cargo")
            .arg("--version")
            .output()
            .context("Cargo toolchain not found. Please install Rust and Cargo.")?;
        Ok(())
    }

    /// Run `cargo build`.
    ///
    /// # Complexity
    /// - Depends on the Cargo build process; runtime varies.
    fn build(&self, package: &str, target: &str, release: bool) -> Result<()> {
        let mut args: Vec<&str> = vec!["build", "--target", target, "-p", package];
        if release {
            args.push("--release");
        }

        // Run cargo build.
        let status = Command::new("cargo")
            .args(args)
            .status()
            .context("Failed to spawn cargo build process")?;

        if !status.success() {
            anyhow::bail!("cargo build failed with non-zero exit code");
        }

        Ok(())
    }

    /// Locate Cargo-produced Wasm output.
    ///
    /// # Logic
    /// - Try common naming conventions (crate_name.wasm, libcrate_name.wasm, etc.).
    /// - If not found, scan all .wasm files in the target directory.
    fn find_output(&self, package: &str, target: &str, release: bool) -> Result<PathBuf> {
        let profile_dir = if release { "release" } else { "debug" };
        let dir = Path::new("target").join(target).join(profile_dir);

        if !dir.exists() {
            anyhow::bail!("Target directory does not exist: {}", dir.display());
        }

        // Replace '-' with '_' to follow Rust crate naming rules.
        let crate_name = package.replace('-', "_");

        // Common output file name candidates.
        let candidates = [
            format!("{crate_name}.wasm"),
            format!("lib{crate_name}.wasm"),
            format!("{package}.wasm"),
            format!("lib{package}.wasm"),
        ];

        // Strategy 1: exact match on common names.
        for name in candidates {
            let p = dir.join(&name);
            if p.exists() {
                return Ok(p);
            }
        }

        // Strategy 2: scan directory for wasm files containing crate_name.
        let rd = std::fs::read_dir(&dir)
            .with_context(|| format!("Failed to read dir: {}", dir.display()))?;

        let mut wasm_files = Vec::new();
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("wasm") {
                wasm_files.push(p);
            }
        }

        if let Some(found) = wasm_files.iter().find(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.contains(&crate_name))
                .unwrap_or(false)
        }) {
            return Ok(found.to_path_buf());
        }

        anyhow::bail!(
            "Wasm output not found in: {} (tried common names + scan)",
            dir.display()
        );
    }
}
