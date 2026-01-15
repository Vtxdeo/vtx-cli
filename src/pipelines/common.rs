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

pub fn resolve_sdk_version(language: &str) -> Option<String> {
    if language.eq_ignore_ascii_case("rust") || language.eq_ignore_ascii_case("rs") {
        checker::read_rust_sdk_version(Path::new("."))
    } else {
        None
    }
}

pub fn build_vtx_metadata_json(
    package_name: &str,
    language: &str,
    project_info: Option<&config::ProjectInfo>,
    sdk_version: Option<&str>,
) -> Result<Vec<u8>> {
    let author = project_info.and_then(|p| p.author.as_deref());
    let authors = project_info.and_then(|p| p.authors.as_ref());
    let description = project_info.and_then(|p| p.description.as_deref());
    let license = project_info.and_then(|p| p.license.as_deref());
    let homepage = project_info.and_then(|p| p.homepage.as_deref());
    let repository = project_info.and_then(|p| p.repository.as_deref());
    let keywords = project_info.and_then(|p| p.keywords.as_ref());
    let version = project_info.and_then(|p| p.version.as_deref());

    let meta = json!({
        "schema": 1,
        "author": author,
        "authors": authors,
        "description": description,
        "license": license,
        "homepage": homepage,
        "repository": repository,
        "keywords": keywords,
        "version": version,
        "sdk_version": sdk_version,
        "package": package_name,
        "language": language,
        "tool": { "name": "vtx-cli", "version": env!("CARGO_PKG_VERSION") }
    });

    Ok(serde_json::to_vec(&meta)?)
}
