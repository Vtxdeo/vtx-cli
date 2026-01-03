use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use toml::Table;

/// 检查 Rust 项目的 SDK 依赖兼容性
///
/// 职责：
/// 读取项目根目录下的 Cargo.toml，解析其 `vtx-sdk` 依赖版本，
/// 并与 CLI 内置的 SDK 元数据版本进行比对。
///
/// 行为：
/// - 若 Cargo.toml 不存在，静默跳过（非 Rust 项目）。
/// - 若发现版本不兼容：
///   - 默认抛出错误终止构建。
///   - 若 `force` 为 true，则仅打印警告日志。
pub fn check_rust_sdk_version(project_dir: &Path, force: bool) -> Result<()> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&cargo_toml_path).context("Failed to read Cargo.toml")?;
    let table: Table = toml::from_str(&content)?;

    // 获取依赖中的 vtx-sdk 版本
    // 优先检查 [dependencies]，其次检查 [dev-dependencies]
    let version = table
        .get("dependencies")
        .and_then(|d| d.get("vtx-sdk"))
        .or_else(|| table.get("dev-dependencies").and_then(|d| d.get("vtx-sdk")));

    match version {
        Some(v) => {
            let user_ver = v
                .as_str()
                .or_else(|| v.get("version").and_then(|value| value.as_str()))
                .unwrap_or("unknown");
            let cli_target_ver = vtx_sdk::VERSION; // 来自 SDK 常量

            // 检查版本兼容性
            if !is_compatible(user_ver, cli_target_ver) {
                let msg = format!(
                    "SDK Version Mismatch: Plugin uses vtx-sdk {user_ver}, but this CLI is optimized for v{cli_target_ver}."
                );

                if force {
                    println!("{} {} (Force build enabled)", "[WARN]".yellow(), msg);
                } else {
                    anyhow::bail!(
                        "{msg}\nHint: Update vtx-sdk in Cargo.toml or use --force to bypass."
                    );
                }
            } else {
                println!(
                    "{} SDK compatibility check passed (v{})",
                    "[INFO]".cyan(),
                    user_ver
                );
            }
        }
        None => {
            // 如果是 Rust 项目但没有 vtx-sdk，可能是裸写 Wasm 或间接依赖，发出警告
            println!(
                "{} Warning: 'vtx-sdk' dependency not found in Cargo.toml.",
                "[WARN]".yellow()
            );
        }
    }
    Ok(())
}

/// 读取 Rust 项目中声明的 vtx-sdk 版本（来自 Cargo.toml）。
pub fn read_rust_sdk_version(project_dir: &Path) -> Option<String> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&cargo_toml_path).ok()?;
    let table: Table = toml::from_str(&content).ok()?;

    let version = table
        .get("dependencies")
        .and_then(|d| d.get("vtx-sdk"))
        .or_else(|| table.get("dev-dependencies").and_then(|d| d.get("vtx-sdk")))?;

    let user_ver = version
        .as_str()
        .or_else(|| version.get("version").and_then(|value| value.as_str()))?;

    Some(user_ver.trim_start_matches(['^', '~', '=']).to_string())
}

/// 简易版本兼容性检查
///
/// 逻辑：
/// 移除 semver 的修饰符（^, ~, =）后，要求版本号字符串完全匹配。
fn is_compatible(user: &str, system: &str) -> bool {
    let clean_user = user.trim_start_matches(['^', '~', '=']);
    clean_user == system
}
