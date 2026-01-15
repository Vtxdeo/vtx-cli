use super::Builder;
use crate::config::BuildConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Python 构建器
///
/// 依赖：componentize-py (用于将 Python 转换为 Wasm Component)
pub struct PythonBuilder {
    pub build_config: Option<BuildConfig>,
}

impl PythonBuilder {
    pub fn new(build_config: Option<BuildConfig>) -> Self {
        Self { build_config }
    }
}

impl Builder for PythonBuilder {
    fn check_env(&self) -> Result<()> {
        Command::new("python")
            .arg("--version")
            .output()
            .context("Python not found.")?;

        if self
            .build_config
            .as_ref()
            .and_then(|c| c.cmd.as_ref())
            .is_none()
        {
            Command::new("componentize-py")
                .arg("--help")
                .output()
                .context("componentize-py not found. Please run: pip install componentize-py")?;
        }
        Ok(())
    }

    fn build(&self, package: &str, _target: &str, _release: bool) -> Result<()> {
        // 1. 自定义命令优先
        if let Some(cmd) = self.build_config.as_ref().and_then(|c| c.cmd.as_ref()) {
            println!("[VTX] Executing custom build command: {cmd}");
            let (shell, arg) = if cfg!(target_os = "windows") {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };

            let status = Command::new(shell)
                .args([arg, cmd])
                .status()
                .with_context(|| format!("Failed to execute command: {cmd}"))?;

            if !status.success() {
                anyhow::bail!("Custom build command failed");
            }
            return Ok(());
        }

        // 2. 默认使用 componentize-py
        println!("[VTX] No 'build.cmd' found, defaulting to 'componentize-py'...");

        let output_dir = Path::new("dist");
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let output_file = output_dir.join(format!("{package}.wasm"));

        let module_name = package.replace('-', "_");
        let status = Command::new("componentize-py")
            .arg("-d")
            .arg(".")
            .arg("-o")
            .arg(&output_file)
            .arg(&module_name)
            .status()
            .context(
                "Failed to execute componentize-py. Ensure pip install componentize-py is run.",
            )?;

        if !status.success() {
            anyhow::bail!("componentize-py build failed");
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

            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "wasm") {
                        return Ok(path);
                    }
                }
            }
        }

        let search_dirs = vec!["dist", "build", "target", "."];
        for dir in search_dirs {
            let p = Path::new(dir).join(format!("{package}.wasm"));
            if p.exists() {
                return Ok(p);
            }

            let p_index = Path::new(dir).join("index.wasm");
            if p_index.exists() {
                return Ok(p_index);
            }
        }

        anyhow::bail!("Could not find .wasm output. Please specify 'build.output_dir' in vtx.toml")
    }
}
