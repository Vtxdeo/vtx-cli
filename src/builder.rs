use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// 执行 `cargo build` 命令
///
/// # 参数
/// - `package`: 要构建的包名称
/// - `target`: 构建的目标架构
/// - `release`: 是否启用发布模式（release mode）
///
/// # 错误处理
/// - 如果 `cargo build` 执行失败，会返回一个错误
pub fn cargo_build(package: &str, target: &str, release: bool) -> Result<()> {
    // 构造 cargo build 命令的参数
    let mut args: Vec<&str> = vec!["build", "--target", target, "-p", package];
    if release {
        args.push("--release");
    }

    // 执行 cargo build 命令
    let status = Command::new("cargo")
        .args(args)
        .status()
        .context("failed to spawn cargo build")?; // 如果执行失败，记录错误

    // 检查命令是否成功执行
    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    Ok(())
}

/// 根据 package 名和目标架构定位 wasm 输出文件
///
/// # 参数
/// - `package`: 包的名称
/// - `target`: 构建的目标架构
/// - `release`: 是否为发布模式（release mode）
///
/// # 返回值
/// - 返回找到的 wasm 文件路径，如果没有找到则返回错误
pub fn find_wasm_output(package: &str, target: &str, release: bool) -> Result<PathBuf> {
    // 根据是否是发布模式（release）选择不同的文件夹路径
    let profile_dir = if release { "release" } else { "debug" };
    let dir = Path::new("target").join(target).join(profile_dir);

    // 检查目标文件夹是否存在
    if !dir.exists() {
        anyhow::bail!("Target directory does not exist: {}", dir.display());
    }

    // 格式化包名，替换掉包名中的 '-' 为 '_'
    let crate_name = package.replace('-', "_");

    // 定义常见的输出文件名候选列表
    let candidates = [
        format!("{crate_name}.wasm"),
        format!("lib{crate_name}.wasm"),
        format!("{package}.wasm"),
        format!("lib{package}.wasm"),
    ];

    // 尝试匹配常见的文件名
    for name in candidates {
        let p = dir.join(&name);
        if p.exists() {
            return Ok(p); // 如果文件存在，直接返回
        }
    }

    // 如果常见名称未找到，扫描目录以查找 wasm 文件
    let rd = std::fs::read_dir(&dir)
        .with_context(|| format!("failed to read dir: {}", dir.display()))?;

    // 存储找到的 wasm 文件
    let mut wasm_files = Vec::new();
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("wasm") {
            wasm_files.push(p); // 如果是 wasm 文件，添加到列表中
        }
    }

    // 查找包含 crate_name 的 wasm 文件
    if let Some(found) = wasm_files.iter().find(|p| {
        p.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.contains(&crate_name))
            .unwrap_or(false)
    }) {
        return Ok(found.to_path_buf()); // 如果找到匹配文件，返回路径
    }

    // 如果没有找到匹配的 wasm 文件，返回错误
    anyhow::bail!(
        "wasm output not found in: {} (tried common names + scan)",
        dir.display()
    );
}
