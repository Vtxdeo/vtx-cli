use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use std::time::Instant;

use crate::{builder::create_builder, checker, config, packager};

use super::common::{
    build_vtx_metadata_json, execute_custom_build, resolve_sdk_version, resolve_wasm_path,
};

/// Execute standard build pipeline
///
/// Flow:
/// 1. Initialize config and context
/// 2. SDK compatibility check (for Rust)
/// 3. Environment pre-check
/// 4. Compile source code
/// 5. Resolve artifact path
/// 6. Encode and package VTX component
pub fn execute_build_pipeline(
    package_arg: Option<String>,
    target: &str,
    release: bool,
    force: bool,
    debug: bool,
) -> Result<()> {
    let start_time = Instant::now();

    // --- 1. Initialize Config ---
    let config = config::load().ok(); // Config is optional allows pure CLI usage
    let project_info = config.as_ref().map(|c| c.project.clone());
    let build_config = config.as_ref().and_then(|c| c.build.clone());

    // Package name priority: CLI arg > Config file > Error
    let package_name = package_arg
        .or_else(|| project_info.as_ref().map(|p| p.name.clone()))
        .context("Unable to resolve package name. Please specify via --package or vtx.toml.")?;

    // Language detection: Default to Rust
    let language = project_info
        .as_ref()
        .map(|p| p.language.as_str())
        .unwrap_or("rust");

    println!(
        "{} Building package: {} [{}]",
        "[VTX]".green().bold(),
        package_name,
        language
    );

    // --- 2. SDK Compatibility Check ---
    if language.to_lowercase() == "rust" || language.to_lowercase() == "rs" {
        if debug {
            println!("{} Checking SDK compatibility...", "[DEBUG]".dimmed());
        }
        checker::check_rust_sdk_version(Path::new("."), force)?;
    } else if debug {
        println!(
            "{} Skipping SDK check for non-Rust project.",
            "[DEBUG]".dimmed()
        );
    }

    // Instantiate language-specific builder strategy
    let builder = create_builder(language, build_config.clone())?;

    // --- 3. Environment Pre-check ---
    if build_config.as_ref().and_then(|c| c.cmd.as_ref()).is_none() {
        builder
            .check_env()
            .context("Environment validation failed")?;
    }

    // --- 4. Compilation Stage ---
    // If in debug mode, force non-release build to keep symbols
    let actual_release = if debug {
        println!(
            "{} Debug mode enabled: forcing non-release build.",
            "[INFO]".cyan()
        );
        false
    } else {
        release
    };

    if let Some(cmd) = build_config.as_ref().and_then(|c| c.cmd.as_ref()).cloned() {
        execute_custom_build(&cmd)?;
    } else {
        println!(
            "{} Compiling target: {} (release={})",
            "[INFO]".cyan(),
            target,
            actual_release
        );
        builder
            .build(&package_name, target, actual_release)
            .context("Source compilation failed")?;
    }

    // --- 5. Artifact Resolution ---
    let wasm_path = resolve_wasm_path(
        &package_name,
        target,
        actual_release,
        build_config.as_ref(),
        builder.as_ref(),
    )?;

    println!(
        "{} Artifact located at: {}",
        "[INFO]".cyan(),
        wasm_path.display()
    );

    // --- 6. Encoding and Packaging ---
    println!(
        "{} Encoding and validating VTX component...",
        "[INFO]".cyan()
    );

    // Pass debug and force flags for internal logic control
    let component_bytes = packager::process_wasm(&wasm_path, debug, force)
        .context("Component packaging or validation failed")?;

    let sdk_version = resolve_sdk_version(language);
    let metadata_json = build_vtx_metadata_json(
        &package_name,
        language,
        project_info.as_ref(),
        sdk_version.as_deref(),
    )?;

    let vtx_path = packager::write_vtx_file(&wasm_path, &component_bytes, &metadata_json)
        .context("Failed to write final artifact")?;

    let duration = start_time.elapsed();
    println!(
        "{} Build completed in {:.2}s 鈫?{}",
        "[DONE]".green().bold(),
        duration.as_secs_f64(),
        vtx_path.display()
    );

    Ok(())
}
