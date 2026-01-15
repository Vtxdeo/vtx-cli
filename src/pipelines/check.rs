use anyhow::{Context, Result};
use colored::*;
use std::path::Path;

use crate::{builder::create_builder, checker, config};

pub fn execute_check_pipeline(debug: bool) -> Result<()> {
    let config = config::load()?;
    let project_info = config.project;
    let build_config = config.build;
    let sdk_config = config.sdk;

    let language = project_info.language;

    if let Some(version) = sdk_config.and_then(|sdk| sdk.version) {
        println!("{} SDK version declared: {}", "[INFO]".cyan(), version);
    }

    if language.to_lowercase() == "rust" || language.to_lowercase() == "rs" {
        if debug {
            println!("{} Checking SDK compatibility...", "[DEBUG]".dimmed());
        }
        checker::check_rust_sdk_version(Path::new("."), false)?;
    } else if debug {
        println!(
            "{} Skipping SDK check for non-Rust project.",
            "[DEBUG]".dimmed()
        );
    }

    let builder = create_builder(&language, build_config.clone())?;
    if build_config.as_ref().and_then(|c| c.cmd.as_ref()).is_none() {
        builder
            .check_env()
            .context("Environment validation failed")?;
    }

    println!(
        "{} Environment check passed for language: {}",
        "[OK]".green().bold(),
        language
    );

    Ok(())
}
