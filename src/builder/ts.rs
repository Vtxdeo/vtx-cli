use super::Builder;
use crate::config::BuildConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// TS/JS builder (NPM ecosystem).
///
/// Responsibilities: execute NPM scripts.
/// Dependencies: Node.js and npm must be installed.
pub struct TsBuilder {
    pub build_config: Option<BuildConfig>,
}

impl TsBuilder {
    pub fn new(build_config: Option<BuildConfig>) -> Self {
        Self { build_config }
    }
}

impl Builder for TsBuilder {
    fn check_env(&self) -> Result<()> {
        let npm_cmd = if cfg!(target_os = "windows") {
            "npm.cmd"
        } else {
            "npm"
        };
        Command::new(npm_cmd)
            .arg("-v")
            .output()
            .context("npm not found")?;
        Ok(())
    }

    fn build(&self, _package: &str, _target: &str, _release: bool) -> Result<()> {
        let npm_cmd = if cfg!(target_os = "windows") {
            "npm.cmd"
        } else {
            "npm"
        };

        // 1. Run user-provided custom command first.
        if let Some(cmd) = self.build_config.as_ref().and_then(|c| c.cmd.as_ref()) {
            let (shell, arg) = if cfg!(target_os = "windows") {
                ("cmd", "/C")
            } else {
                ("sh", "-c")
            };
            let status = Command::new(shell).args([arg, cmd]).status()?;
            if !status.success() {
                anyhow::bail!("Custom JS/TS build command failed");
            }
            return Ok(());
        }

        // 2. Ensure dependencies are present (may trigger network IO).
        if Path::new("package.json").exists() && !Path::new("node_modules").exists() {
            println!("[VTX] node_modules not found, running npm install...");
            let status = Command::new(npm_cmd).arg("install").status()?;
            if !status.success() {
                anyhow::bail!("npm install failed");
            }
        }

        // 3. Run standard npm build script.
        println!("[VTX] Executing: {npm_cmd} run build");
        let status = Command::new(npm_cmd).arg("run").arg("build").status()?;

        if !status.success() {
            anyhow::bail!("npm run build failed");
        }

        Ok(())
    }

    fn find_output(&self, package: &str, _target: &str, _release: bool) -> Result<PathBuf> {
        // Strategy 1: use configured output_dir first.
        if let Some(dir) = self
            .build_config
            .as_ref()
            .and_then(|c| c.output_dir.as_ref())
        {
            let p = Path::new(dir).join(format!("{package}.wasm"));
            if p.exists() {
                return Ok(p);
            }

            // Fallback: fuzzy match inside the directory.
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|e| e == "wasm") {
                        return Ok(entry.path());
                    }
                }
            }
        }

        // Strategy 2: heuristic search in standard directories.
        let search_dirs = vec!["build", "dist", "target", "."];
        let candidates = vec![
            format!("{package}.wasm"),
            "release.wasm".to_string(),
            "index.wasm".to_string(),
        ];

        for dir in search_dirs {
            let dir_path = Path::new(dir);
            if !dir_path.exists() {
                continue;
            }
            for name in &candidates {
                let p = dir_path.join(name);
                if p.exists() {
                    return Ok(p);
                }
            }
        }

        anyhow::bail!(
            "Wasm output not found. Please set 'build.output_dir' in vtx.toml or check npm build script."
        )
    }
}
