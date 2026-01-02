use super::Builder;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Go (TinyGo) 语言构建器
///
/// 职责：封装 TinyGo 工具链调用，用于构建 Go 语言编写的插件。
/// 注意：依赖 tinygo 命令行工具，且目标架构通常需指定为 wasi。
pub struct GoBuilder;

impl Builder for GoBuilder {
    /// 检查 tinygo 环境
    fn check_env(&self) -> Result<()> {
        Command::new("tinygo")
            .arg("version")
            .output()
            .context("TinyGo toolchain not found. Please install TinyGo: https://tinygo.org/getting-started/install/")?;
        Ok(())
    }

    /// 执行 tinygo build 命令
    ///
    /// # 副作用
    /// - 在 target 目录下创建构建产物。
    /// - 调用外部进程 tinygo。
    fn build(&self, package: &str, target: &str, release: bool) -> Result<()> {
        // 1. 准备输出目录 (模仿 Rust 的 target 结构以保持一致性)
        let profile = if release { "release" } else { "debug" };
        let output_dir = Path::new("target").join(target).join(profile);

        // 确保目录存在
        fs::create_dir_all(&output_dir)
            .context("Failed to create target directory for Go build")?;

        // 2. 确定输出文件名
        let output_path = output_dir.join(format!("{}.wasm", package));

        // 3. 构造 tinygo 命令
        // 示例: tinygo build -target=wasi -o target/wasm32-wasip1/release/pkg.wasm .
        // 注意: TinyGo 目前使用 'wasi' target 来支持 WASI Preview 1
        let mut args = vec!["build", "-target=wasi", "-o", output_path.to_str().unwrap()];

        if release {
            // TinyGo 特有参数，发布模式去除调试信息
            args.push("-no-debug");
        }

        // 假设当前执行目录即为 Go 项目根目录
        args.push(".");

        println!("[VTX] Executing: tinygo {}", args.join(" "));

        let status = Command::new("tinygo")
            .args(args)
            .status()
            .context("Failed to execute tinygo build process")?;

        if !status.success() {
            anyhow::bail!("tinygo build failed with non-zero exit code");
        }

        Ok(())
    }

    /// 查找 TinyGo 构建产物
    ///
    /// # 逻辑
    /// 由于 build 阶段显式指定了输出路径，此处直接返回该路径并验证文件是否存在。
    fn find_output(&self, package: &str, target: &str, release: bool) -> Result<PathBuf> {
        let profile = if release { "release" } else { "debug" };
        let path = Path::new("target")
            .join(target)
            .join(profile)
            .join(format!("{}.wasm", package));

        if path.exists() {
            Ok(path)
        } else {
            anyhow::bail!("Expected build artifact not found at: {}", path.display())
        }
    }
}
