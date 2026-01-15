use anyhow::{Context, Result};
use colored::*;
use std::path::Path;

use crate::{config, packager};

use super::common::{build_vtx_metadata_json, resolve_sdk_version};

pub fn execute_package_pipeline(input: &str, debug: bool, force: bool) -> Result<()> {
    let wasm_path = Path::new(input);
    if !wasm_path.exists() {
        anyhow::bail!("Input file not found: {}", wasm_path.display());
    }

    println!(
        "{} Packaging input: {}",
        "[INFO]".cyan(),
        wasm_path.display()
    );

    let component_bytes = packager::process_wasm(wasm_path, debug, force)
        .context("Component packaging or validation failed")?;

    let config = config::load().ok();
    let package_name = config
        .as_ref()
        .map(|c| c.project.name.clone())
        .or_else(|| {
            wasm_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    let language = config
        .as_ref()
        .map(|c| c.project.language.as_str())
        .unwrap_or("unknown");
    let author = config.as_ref().and_then(|c| c.project.author.clone());
    let sdk_version = resolve_sdk_version(language, config.as_ref());
    let metadata_json = build_vtx_metadata_json(
        &package_name,
        language,
        author.as_deref(),
        sdk_version.as_deref(),
    )?;

    let vtx_path = packager::write_vtx_file(wasm_path, &component_bytes, &metadata_json)
        .context("Failed to write final artifact")?;

    println!(
        "{} Package completed 鈫?{}",
        "[DONE]".green().bold(),
        vtx_path.display()
    );

    Ok(())
}