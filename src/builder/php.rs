use super::Builder;
use crate::config::BuildConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// PHP builder.
///
/// Dependencies: Composer (recommended) or user-defined scripts.
pub struct PhpBuilder {
    pub build_config: Option<BuildConfig>,
}

impl PhpBuilder {
    pub fn new(build_config: Option<BuildConfig>) -> Self {
        Self { build_config }
    }
}

impl Builder for PhpBuilder {
    fn check_env(&self) -> Result<()> {
        Command::new("php")
            .arg("-v")
            .output()
            .context("PHP runtime not found.")?;
        Ok(())
    }

    fn build(&self, _package: &str, _target: &str, _release: bool) -> Result<()> {
        // 1. Custom command takes priority.
        if let Some(cmd) = self.build_config.as_ref().and_then(|c| c.cmd.as_ref()) {
            let (shell, arg) = if cfg!(target_os = "windows") {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };
            let status = Command::new(shell).args([arg, cmd]).status()?;
            if !status.success() {
                anyhow::bail!("Custom PHP build command failed");
            }
            return Ok(());
        }

        // 2. Default behavior: run composer build script.
        let composer = if cfg!(target_os = "windows") {
            "composer.bat"
        } else {
            "composer"
        };
        println!("[VTX] Executing 'composer run build'...");

        let status = Command::new(composer)
            .arg("run")
            .arg("build")
            .status()
            .context(
            "Failed to run 'composer run build'. Please define 'scripts.build' in composer.json",
        )?;

        if !status.success() {
            anyhow::bail!("Composer build script failed");
        }
        Ok(())
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

        let candidates = vec![
            Path::new("build").join(format!("{package}.wasm")),
            Path::new("dist").join(format!("{package}.wasm")),
            Path::new("target").join(format!("{package}.wasm")),
            Path::new(".").join(format!("{package}.wasm")),
        ];

        for p in candidates {
            if p.exists() {
                return Ok(p);
            }
        }

        anyhow::bail!("PHP Wasm artifact not found. Please check your build script output.")
    }
}
