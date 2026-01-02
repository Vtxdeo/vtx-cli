use super::Builder;
use crate::config::BuildConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// PHP 构建器
///
/// 依赖：Composer (推荐) 或用户自定义脚本。
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
        // 1. 自定义命令优先
        if let Some(cmd) = self
            .build_config
            .as_ref()
            .and_then(|c| c.cmd.as_ref())
        {
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

        // 2. 默认行为：执行 composer run build
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
            let p = Path::new(dir).join(format!("{}.wasm", package));
            if p.exists() {
                return Ok(p);
            }
        }

        let candidates = vec![
            Path::new("build").join(format!("{}.wasm", package)),
            Path::new("dist").join(format!("{}.wasm", package)),
            Path::new("target").join(format!("{}.wasm", package)),
            Path::new(".").join(format!("{}.wasm", package)),
        ];

        for p in candidates {
            if p.exists() {
                return Ok(p);
            }
        }

        anyhow::bail!("PHP Wasm artifact not found. Please check your build script output.")
    }
}
