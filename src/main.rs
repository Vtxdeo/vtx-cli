mod builder;
mod checker;
mod cli;
mod config;
mod packager;

use anyhow::{Context, Result};
use builder::{create_builder, Builder};
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
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
        Commands::Check { debug } => execute_check_pipeline(debug),
        Commands::Package {
            input,
            force,
            debug,
        } => execute_package_pipeline(&input, debug, force),
        Commands::Init { name, language } => execute_init_pipeline(&name, &language),
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
    let build_config = config.as_ref().and_then(|c| c.build.clone());

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
    let builder = create_builder(language, build_config.clone())?;

    // --- 3. 环境预检 ---
    if build_config.as_ref().and_then(|c| c.cmd.as_ref()).is_none() {
        builder
            .check_env()
            .context("Environment validation failed")?;
    }

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

    if let Some(cmd) = build_config.as_ref().and_then(|c| c.cmd.as_ref()).cloned() {
        execute_custom_build(&cmd)?;
    } else {
        println!(
            "{} Compiling target: {} (release={})",
            "[INFO]".cyan(),
            target,
            actual_release
        );
        builder
            .build(&package_name, target, actual_release)
            .context("Source compilation failed")?;
    }

    // --- 5. 产物路径解析 ---
    let wasm_path = resolve_wasm_path(
        &package_name,
        target,
        actual_release,
        build_config.as_ref(),
        builder.as_ref(),
    )?;

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

    let author = project_info.as_ref().and_then(|p| p.author.clone());
    let sdk_version = resolve_sdk_version(language, config.as_ref());
    let metadata_json = build_vtx_metadata_json(
        &package_name,
        language,
        author.as_deref(),
        sdk_version.as_deref(),
    )?;

    let vtx_path = packager::write_vtx_file(&wasm_path, &component_bytes, &metadata_json)
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

fn execute_check_pipeline(debug: bool) -> Result<()> {
    let config = config::load()?;
    let project_info = config.project;
    let build_config = config.build;
    let sdk_config = config.sdk;

    let language = project_info.language;

    if let Some(version) = sdk_config.and_then(|sdk| sdk.version) {
        println!("{} SDK version declared: {}", "[INFO]".cyan(), version);
    }

    if language.to_lowercase() == "rust" || language.to_lowercase() == "rs" {
        if debug {
            println!("{} Checking SDK compatibility...", "[DEBUG]".dimmed());
        }
        checker::check_rust_sdk_version(Path::new("."), false)?;
    } else if debug {
        println!(
            "{} Skipping SDK check for non-Rust project.",
            "[DEBUG]".dimmed()
        );
    }

    let builder = create_builder(&language, build_config.clone())?;
    if build_config.as_ref().and_then(|c| c.cmd.as_ref()).is_none() {
        builder
            .check_env()
            .context("Environment validation failed")?;
    }

    println!(
        "{} Environment check passed for language: {}",
        "[OK]".green().bold(),
        language
    );

    Ok(())
}

fn execute_package_pipeline(input: &str, debug: bool, force: bool) -> Result<()> {
    let wasm_path = Path::new(input);
    if !wasm_path.exists() {
        anyhow::bail!("Input file not found: {}", wasm_path.display());
    }

    println!(
        "{} Packaging input: {}",
        "[INFO]".cyan(),
        wasm_path.display()
    );

    let component_bytes = packager::process_wasm(wasm_path, debug, force)
        .context("Component packaging or validation failed")?;

    let config = config::load().ok();
    let package_name = config
        .as_ref()
        .map(|c| c.project.name.clone())
        .or_else(|| {
            wasm_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());
    let language = config
        .as_ref()
        .map(|c| c.project.language.as_str())
        .unwrap_or("unknown");
    let author = config.as_ref().and_then(|c| c.project.author.clone());
    let sdk_version = resolve_sdk_version(language, config.as_ref());
    let metadata_json = build_vtx_metadata_json(
        &package_name,
        language,
        author.as_deref(),
        sdk_version.as_deref(),
    )?;

    let vtx_path = packager::write_vtx_file(wasm_path, &component_bytes, &metadata_json)
        .context("Failed to write final artifact")?;

    println!(
        "{} Package completed → {}",
        "[DONE]".green().bold(),
        vtx_path.display()
    );

    Ok(())
}

fn execute_init_pipeline(name: &str, language: &str) -> Result<()> {
    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Target directory already exists: {}", project_dir.display());
    }

    std::fs::create_dir_all(project_dir)?;

    match language.to_lowercase().as_str() {
        "rust" | "rs" => init_rust(project_dir, name),
        "ts" | "typescript" | "js" | "node" => init_ts(project_dir, name),
        "py" | "python" => init_python(project_dir, name),
        unsupported => anyhow::bail!("Unsupported language identifier: {unsupported}"),
    }?;

    println!(
        "{} Project initialized at: {}",
        "[DONE]".green().bold(),
        project_dir.display()
    );

    Ok(())
}

fn init_rust(project_dir: &Path, name: &str) -> Result<()> {
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let crate_name = name.replace('-', "_");
    let cargo_toml = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nvtx-sdk = \"0.1.8\"\nserde = {{ version = \"1.0\", features = [\"derive\"] }}\nserde_json = \"1.0\"\nanyhow = \"1.0\"\n"
    );

    let lib_rs = "use vtx_sdk::{export_plugin, VtxPlugin};\n\n#[derive(Default)]\nstruct Plugin;\n\nimpl VtxPlugin for Plugin {}\n\nexport_plugin!(Plugin);\n".to_string();

    let vtx_toml = format!(
        "[project]\nname = \"{name}\"\nlanguage = \"rust\"\n\n[build]\ncmd = \"cargo build --target wasm32-wasip1 --release\"\noutput_dir = \"target/wasm32-wasip1/release\"\nartifact = \"{crate_name}.wasm\"\n\n[sdk]\nversion = \"0.1.8\"\n"
    );

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    std::fs::write(src_dir.join("lib.rs"), lib_rs)?;
    std::fs::write(project_dir.join("vtx.toml"), vtx_toml)?;

    Ok(())
}

fn init_ts(project_dir: &Path, name: &str) -> Result<()> {
    let src_dir = project_dir.join("src");
    let dist_dir = project_dir.join("dist");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(&dist_dir)?;

    let package_json = format!(
        "{{\n  \"name\": \"{name}\",\n  \"version\": \"0.1.0\",\n  \"scripts\": {{\n    \"build\": \"echo TODO: build wasm\"\n  }}\n}}\n"
    );

    let index_ts = "export {};\n".to_string();

    let vtx_toml = format!(
        "[project]\nname = \"{name}\"\nlanguage = \"ts\"\n\n[build]\ncmd = \"npm run build\"\noutput_dir = \"dist\"\nartifact = \"{name}.wasm\"\n\n[sdk]\nversion = \"0.2.0\"\n"
    );

    std::fs::write(project_dir.join("package.json"), package_json)?;
    std::fs::write(src_dir.join("index.ts"), index_ts)?;
    std::fs::write(project_dir.join("vtx.toml"), vtx_toml)?;

    Ok(())
}

fn init_python(project_dir: &Path, name: &str) -> Result<()> {
    let src_dir = project_dir.join("src");
    let module_dir = src_dir.join(name.replace('-', "_"));
    let dist_dir = project_dir.join("dist");
    std::fs::create_dir_all(&module_dir)?;
    std::fs::create_dir_all(&dist_dir)?;

    let pyproject = format!(
        "[build-system]\nrequires = [\"setuptools\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[project]\nname = \"{name}\"\nversion = \"0.1.0\"\n"
    );

    let init_py = "# plugin entry\n".to_string();

    let module_name = name.replace('-', "_");
    let vtx_toml = format!(
        "[project]\nname = \"{name}\"\nlanguage = \"python\"\n\n[build]\ncmd = \"componentize-py -d . -o dist/{name}.wasm {module_name}\"\noutput_dir = \"dist\"\nartifact = \"{name}.wasm\"\n\n[sdk]\nversion = \"0.2.0\"\n"
    );

    std::fs::write(project_dir.join("pyproject.toml"), pyproject)?;
    std::fs::write(module_dir.join("__init__.py"), init_py)?;
    std::fs::write(project_dir.join("vtx.toml"), vtx_toml)?;

    Ok(())
}

fn execute_custom_build(cmd: &str) -> Result<()> {
    let (shell, arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let status = Command::new(shell)
        .args([arg, cmd])
        .status()
        .with_context(|| format!("Failed to execute build command: {cmd}"))?;

    if !status.success() {
        anyhow::bail!("Custom build command failed");
    }

    Ok(())
}

fn resolve_wasm_path(
    package: &str,
    target: &str,
    release: bool,
    build_config: Option<&config::BuildConfig>,
    builder: &dyn Builder,
) -> Result<PathBuf> {
    if let Some(cfg) = build_config {
        if let (Some(dir), Some(artifact)) = (cfg.output_dir.as_ref(), cfg.artifact.as_ref()) {
            let path = Path::new(dir).join(artifact);
            if path.exists() {
                return Ok(path);
            }
            anyhow::bail!("Expected artifact not found at: {}", path.display());
        }
    }

    builder
        .find_output(package, target, release)
        .context("Unable to locate compiled artifact")
}

fn resolve_sdk_version(language: &str, config: Option<&config::ProjectConfig>) -> Option<String> {
    let declared = config
        .and_then(|c| c.sdk.as_ref())
        .and_then(|s| s.version.clone());

    if language.eq_ignore_ascii_case("rust") || language.eq_ignore_ascii_case("rs") {
        checker::read_rust_sdk_version(Path::new(".")).or(declared)
    } else {
        declared
    }
}

fn build_vtx_metadata_json(
    package_name: &str,
    language: &str,
    author: Option<&str>,
    sdk_version: Option<&str>,
) -> Result<Vec<u8>> {
    let meta = json!({
        "schema": 1,
        "author": author,
        "sdk_version": sdk_version,
        "package": package_name,
        "language": language,
        "tool": { "name": "vtx-cli", "version": env!("CARGO_PKG_VERSION") }
    });

    Ok(serde_json::to_vec(&meta)?)
}
