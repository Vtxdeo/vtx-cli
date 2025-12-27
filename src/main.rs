mod builder;
mod checker;
mod cli;
mod config;
mod packager;

use anyhow::{Context, Result};
use builder::Builder;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use std::path::Path;
use std::time::Instant;

/// CLI 主入口
fn main() -> Result<()> {
    let cli = Cli::parse();

    // 捕获顶层错误，格式化输出，避免展示 Rust 栈信息
    if let Err(e) = run(cli) {
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

/// 执行业务主流程
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Build {
            package,
            target,
            release,
            force,
            debug,
        } => execute_build_pipeline(package, &target, release, force, debug),
    }
}

/// 执行标准构建流水线
///
/// 流程结构：
/// 1. 初始化配置与上下文
/// 2. SDK 兼容性检查 (针对 Rust)
/// 3. 环境预检
/// 4. 编译源代码
/// 5. 产物路径解析
/// 6. 编码打包为 VTX 组件并校验
fn execute_build_pipeline(
    package_arg: Option<String>,
    target: &str,
    release: bool,
    force: bool,
    debug: bool,
) -> Result<()> {
    let start_time = Instant::now();

    // --- 1. 初始化配置 ---
    let config = config::load().ok(); // 配置可选，允许纯 CLI 模式
    let project_info = config.as_ref().map(|c| c.project.clone());

    // 包名优先级：命令行 > 配置文件 > 报错
    let package_name = package_arg
        .or_else(|| project_info.as_ref().map(|p| p.name.clone()))
        .context("Unable to resolve package name. Please specify via --package or vtx.toml.")?;

    // 语言识别：默认使用 Rust
    let language = project_info
        .as_ref()
        .map(|p| p.language.as_str())
        .unwrap_or("rust");

    println!(
        "{} Building package: {} [{}]",
        "[VTX]".green().bold(),
        package_name,
        language
    );

    // --- 2. SDK 兼容性检查 ---
    if language.to_lowercase() == "rust" || language.to_lowercase() == "rs" {
        if debug {
            println!("{} Checking SDK compatibility...", "[DEBUG]".dimmed());
        }
        checker::check_rust_sdk_version(Path::new("."), force)?;
    } else if debug {
        println!(
            "{} Skipping SDK check for non-Rust project.",
            "[DEBUG]".dimmed()
        );
    }

    // 实例化对应语言的构建器策略
    let builder: Box<dyn Builder> = match language.to_lowercase().as_str() {
        "rust" | "rs" => Box::new(builder::rust::RustBuilder),
        "go" | "tinygo" => Box::new(builder::go::GoBuilder),
        "ts" | "typescript" | "js" | "node" => Box::new(builder::ts::TsBuilder::new(project_info)),
        "py" | "python" => Box::new(builder::python::PythonBuilder::new(project_info)),
        "php" => Box::new(builder::php::PhpBuilder::new(project_info)),
        "lua" => Box::new(builder::lua::LuaBuilder::new(project_info)),
        unsupported => anyhow::bail!("Unsupported language identifier: {}", unsupported),
    };

    // --- 3. 环境预检 ---
    builder
        .check_env()
        .context("Environment validation failed")?;

    // --- 4. 编译阶段 ---
    // 如果处于 debug 模式，强制编译为 debug 版本以保留符号表
    let actual_release = if debug {
        println!(
            "{} Debug mode enabled: forcing non-release build.",
            "[INFO]".cyan()
        );
        false
    } else {
        release
    };

    println!(
        "{} Compiling target: {} (release={})",
        "[INFO]".cyan(),
        target,
        actual_release
    );
    builder
        .build(&package_name, target, actual_release)
        .context("Source compilation failed")?;

    // --- 5. 产物路径解析 ---
    let wasm_path = builder
        .find_output(&package_name, target, actual_release)
        .context("Unable to locate compiled artifact")?;

    println!(
        "{} Artifact located at: {}",
        "[INFO]".cyan(),
        wasm_path.display()
    );

    // --- 6. 编码与组件打包 ---
    println!(
        "{} Encoding and validating VTX component...",
        "[INFO]".cyan()
    );

    // 传入 debug 和 force 参数进行内部逻辑控制
    let component_bytes = packager::process_wasm(&wasm_path, debug, force)
        .context("Component packaging or validation failed")?;

    let vtx_path = packager::write_vtx_file(&wasm_path, &component_bytes)
        .context("Failed to write final artifact")?;

    let duration = start_time.elapsed();
    println!(
        "{} Build completed in {:.2}s → {}",
        "[DONE]".green().bold(),
        duration.as_secs_f64(),
        vtx_path.display()
    );

    Ok(())
}
