mod builder;
mod cli;
mod packager;

use anyhow::Result;
use clap::Parser;
use colored::*;
use cli::{Cli, Commands};

/// 主程序入口，解析命令行参数并调用相应的处理函数
fn main() -> Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 根据命令类型调用相应的处理函数
    match cli.command {
        Commands::Build {
            package,
            target,
            release,
        } => handle_build(&package, &target, release),
    }
}

/// 处理构建命令
///
/// # 参数
/// - `package`: 构建的包名
/// - `target`: 构建的目标架构
/// - `release`: 是否启用发布模式（release mode）
///
/// # 错误处理
/// - 构建过程中任何步骤失败都会返回错误
fn handle_build(package: &str, target: &str, release: bool) -> Result<()> {
    // 输出构建信息
    println!(
        "{} Starting build for package: {}",
        "[VTX]".green(),
        package.yellow()
    );
    println!("{} Target: {}, Release: {}", "[VTX]".dimmed(), target, release);

    // 1. 执行 Cargo 编译
    println!("{}", "[VTX] Compiling to WebAssembly...".cyan());
    builder::cargo_build(package, target, release)?;

    // 2. 查找构建产物（WebAssembly 文件）
    let wasm_path = builder::find_wasm_output(package, target, release)?;
    println!(
        "{} Found artifact: {}",
        "[VTX]".green(),
        wasm_path.display().to_string().yellow()
    );

    // 3. 打包处理：Strip -> Adapter -> Component
    println!("{}", "[VTX] Packaging VTX plugin...".cyan());
    let component_bytes = packager::process_wasm(&wasm_path)?;

    // 4. 写入最终格式文件
    let vtx_path = packager::write_vtx_file(&wasm_path, &component_bytes)?;
    println![
        "{} VTX package generated successfully:\n      {}",
        "[VTX]".green(),
        vtx_path.display().to_string().yellow()
    ];

    Ok(())
}
