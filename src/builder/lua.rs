use super::Builder;
use crate::config::BuildConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Lua builder.
///
/// Note: the Lua ecosystem lacks a standard Wasm build flow, so it relies on
/// user-provided build.cmd.
pub struct LuaBuilder {
    pub build_config: Option<BuildConfig>,
}

impl LuaBuilder {
    pub fn new(build_config: Option<BuildConfig>) -> Self {
        Self { build_config }
    }
}

impl Builder for LuaBuilder {
    fn check_env(&self) -> Result<()> {
        Command::new("lua")
            .arg("-v")
            .output()
            .context("Lua interpreter not found.")?;
        Ok(())
    }

    fn build(&self, _package: &str, _target: &str, _release: bool) -> Result<()> {
        // 1. Custom command is required if provided.
        if let Some(cmd) = self.build_config.as_ref().and_then(|c| c.cmd.as_ref()) {
            let (shell, arg) = if cfg!(target_os = "windows") {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };
            let status = Command::new(shell).args([arg, cmd]).status()?;
            if !status.success() {
                anyhow::bail!("Custom Lua build command failed");
            }
            return Ok(());
        }

        // 2. Fallback: check for Makefile.
        if Path::new("Makefile").exists() {
            println!("[VTX] Makefile detected, running 'make'...");
            let status = Command::new("make")
                .status()
                .context("Failed to run make")?;
            if !status.success() {
                anyhow::bail!("Make execution failed");
            }
            return Ok(());
        }

        anyhow::bail!("No build method found for Lua. Please specify 'build.cmd' in vtx.toml")
    }

    fn find_output(&self, package: &str, _target: &str, _release: bool) -> Result<PathBuf> {
        if let Some(dir) = self
            .build_config
            .as_ref()
            .and_then(|c| c.output_dir.as_ref())
        {
            let p = Path::new(dir).join(format!("{package}.wasm"));
            if p.exists() {
                return Ok(p);
            }
        }

        let p = Path::new(".").join(format!("{package}.wasm"));
        if p.exists() {
            return Ok(p);
        }

        anyhow::bail!("Lua Wasm artifact not found. Please specify 'build.output_dir' in vtx.toml")
    }
}
