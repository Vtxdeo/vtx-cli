use anyhow::{Context, Result};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{builder::Builder, checker, config};

pub fn execute_custom_build(cmd: &str) -> Result<()> {
    let (shell, arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let status = Command::new(shell)
        .args([arg, cmd])
        .status()
        .with_context(|| format!("Failed to execute build command: {cmd}"))?;

    if !status.success() {
        anyhow::bail!("Custom build command failed");
    }

    Ok(())
}

pub fn resolve_wasm_path(
    package: &str,
    target: &str,
    release: bool,
    build_config: Option<&config::BuildConfig>,
    builder: &dyn Builder,
) -> Result<PathBuf> {
    if let Some(cfg) = build_config {
        if let (Some(dir), Some(artifact)) = (cfg.output_dir.as_ref(), cfg.artifact.as_ref()) {
            let path = Path::new(dir).join(artifact);
            if path.exists() {
                return Ok(path);
            }
            anyhow::bail!("Expected artifact not found at: {}", path.display());
        }
    }

    builder
        .find_output(package, target, release)
        .context("Unable to locate compiled artifact")
}

pub fn resolve_sdk_version(language: &str, config: Option<&config::ProjectConfig>) -> Option<String> {
    let declared = config
        .and_then(|c| c.sdk.as_ref())
        .and_then(|s| s.version.clone());

    if language.eq_ignore_ascii_case("rust") || language.eq_ignore_ascii_case("rs") {
        checker::read_rust_sdk_version(Path::new(".")).or(declared)
    } else {
        declared
    }
}

pub fn build_vtx_metadata_json(
    package_name: &str,
    language: &str,
    author: Option<&str>,
    sdk_version: Option<&str>,
) -> Result<Vec<u8>> {
    let meta = json!({
        "schema": 1,
        "author": author,
        "sdk_version": sdk_version,
        "package": package_name,
        "language": language,
        "tool": { "name": "vtx-cli", "version": env!("CARGO_PKG_VERSION") }
    });

    Ok(serde_json::to_vec(&meta)?)
}