use super::Builder;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rust 语言构建器
///
/// 职责：封装 Cargo 工具链的调用逻辑，用于构建 Rust 编写的插件。
pub struct RustBuilder;

impl Builder for RustBuilder {
    /// 检查 cargo 工具链是否可用
    fn check_env(&self) -> Result<()> {
        Command::new("cargo")
            .arg("--version")
            .output()
            .context("Cargo toolchain not found. Please install Rust and Cargo.")?;
        Ok(())
    }

    /// 执行 `cargo build` 命令
    ///
    /// # 复杂度
    /// - 依赖于 Cargo 构建过程，时间复杂度不定。
    fn build(&self, package: &str, target: &str, release: bool) -> Result<()> {
        let mut args: Vec<&str> = vec!["build", "--target", target, "-p", package];
        if release {
            args.push("--release");
        }

        // 执行 cargo build 命令
        let status = Command::new("cargo")
            .args(args)
            .status()
            .context("Failed to spawn cargo build process")?;

        if !status.success() {
            anyhow::bail!("cargo build failed with non-zero exit code");
        }

        Ok(())
    }

    /// 定位 Cargo 构建生成的 Wasm 文件
    ///
    /// # 逻辑
    /// - 首先尝试常见的命名规则（crate_name.wasm, libcrate_name.wasm 等）。
    /// - 如果未找到，扫描目标目录下的所有 .wasm 文件。
    fn find_output(&self, package: &str, target: &str, release: bool) -> Result<PathBuf> {
        let profile_dir = if release { "release" } else { "debug" };
        let dir = Path::new("target").join(target).join(profile_dir);

        if !dir.exists() {
            anyhow::bail!("Target directory does not exist: {}", dir.display());
        }

        // 将包名中的 '-' 替换为 '_'，符合 Rust crate 命名规范
        let crate_name = package.replace('-', "_");

        // 定义常见的输出文件名候选列表
        let candidates = [
            format!("{crate_name}.wasm"),
            format!("lib{crate_name}.wasm"),
            format!("{package}.wasm"),
            format!("lib{package}.wasm"),
        ];

        // 策略 1: 尝试精确匹配常见文件名
        for name in candidates {
            let p = dir.join(&name);
            if p.exists() {
                return Ok(p);
            }
        }

        // 策略 2: 扫描目录查找包含 crate_name 的 wasm 文件
        let rd = std::fs::read_dir(&dir)
            .with_context(|| format!("Failed to read dir: {}", dir.display()))?;

        let mut wasm_files = Vec::new();
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("wasm") {
                wasm_files.push(p);
            }
        }

        if let Some(found) = wasm_files.iter().find(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.contains(&crate_name))
                .unwrap_or(false)
        }) {
            return Ok(found.to_path_buf());
        }

        anyhow::bail!(
            "Wasm output not found in: {} (tried common names + scan)",
            dir.display()
        );
    }
}
