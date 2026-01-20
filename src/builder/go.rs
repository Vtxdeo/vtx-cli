use super::Builder;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Go (TinyGo) builder.
///
/// Responsibilities: wrap TinyGo toolchain calls to build Go plugins.
/// Note: requires the tinygo CLI and usually targets wasi.
pub struct GoBuilder;

impl Builder for GoBuilder {
    /// Check tinygo environment.
    fn check_env(&self) -> Result<()> {
        Command::new("tinygo")
            .arg("version")
            .output()
            .context("TinyGo toolchain not found. Please install TinyGo: https://tinygo.org/getting-started/install/")?;
        Ok(())
    }

    /// Run tinygo build.
    ///
    /// # Side effects
    /// - Creates build artifacts under target.
    /// - Invokes the external tinygo process.
    fn build(&self, package: &str, target: &str, release: bool) -> Result<()> {
        // 1. Prepare output directory (mirror Rust target layout).
        let profile = if release { "release" } else { "debug" };
        let output_dir = Path::new("target").join(target).join(profile);

        // Ensure the directory exists.
        fs::create_dir_all(&output_dir)
            .context("Failed to create target directory for Go build")?;

        // 2. Determine output file name.
        let output_path = output_dir.join(format!("{package}.wasm"));

        // 3. Build tinygo command.
        // Example: tinygo build -target=wasi -o target/wasm32-wasip1/release/pkg.wasm .
        // Note: TinyGo currently uses 'wasi' to target WASI Preview 1.
        let mut args = vec!["build", "-target=wasi", "-o", output_path.to_str().unwrap()];

        if release {
            // TinyGo-specific flag to strip debug info in release builds.
            args.push("-no-debug");
        }

        // Assume current working directory is the Go project root.
        args.push(".");

        println!("[VTX] Executing: tinygo {}", args.join(" "));

        let status = Command::new("tinygo")
            .args(args)
            .status()
            .context("Failed to execute tinygo build process")?;

        if !status.success() {
            anyhow::bail!("tinygo build failed with non-zero exit code");
        }

        Ok(())
    }

    /// Locate the TinyGo build artifact.
    ///
    /// # Logic
    /// Since build specifies the output path, return it and verify it exists.
    fn find_output(&self, package: &str, target: &str, release: bool) -> Result<PathBuf> {
        let profile = if release { "release" } else { "debug" };
        let path = Path::new("target")
            .join(target)
            .join(profile)
            .join(format!("{package}.wasm"));

        if path.exists() {
            Ok(path)
        } else {
            anyhow::bail!("Expected build artifact not found at: {}", path.display())
        }
    }
}
