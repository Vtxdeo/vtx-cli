// vtx-cli/src/main.rs
//
// 目标：
// 1) build 子命令：编译指定插件 crate 到 wasm32-wasip1
// 2) 自动找到正确的 wasm 产物（处理 crate 名 '-' -> '_'）
// 3) 修复 wit-bindgen 的 duplicate import 合并问题：剥离 exports-removed 元数据段
// 4) 注入 WASI preview1 adapter（自动选择 command/reactor）
// 5) 只输出 .vtx（强制唯一产物格式）
//    - .vtx 由 vtx-format 统一编码：VTX\x01 + component bytes

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use wasmparser::{Chunk, Parser as WasmParser, Payload};
use wit_component::ComponentEncoder;

use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME,
    WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER,
    WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 构建并打包插件（wasm -> component -> .vtx）
    Build {
        /// workspace package 名（例如：vtx-plugin-auth-basic）
        #[arg(short, long)]
        package: String,

        /// 构建目标（默认 wasm32-wasip1）
        #[arg(long, default_value = "wasm32-wasip1")]
        target: String,

        /// 是否使用 release（默认 true）
        #[arg(long, default_value_t = true)]
        release: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            package,
            target,
            release,
        } => handle_build(&package, &target, release),
    }
}

fn handle_build(package: &str, target: &str, release: bool) -> Result<()> {
    println!(
        "{} Starting build for package: {}",
        "[VTX]".green(),
        package.yellow()
    );

    println!(
        "{} Compiling to WebAssembly ({})...",
        "[VTX]".cyan(),
        target.yellow()
    );

    // 1) cargo build
    let mut args: Vec<&str> = vec!["build", "--target", target, "-p", package];
    if release {
        args.push("--release");
    }

    let status = Command::new("cargo")
        .args(args)
        .status()
        .context("failed to spawn cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    // 2) 找到 wasm 产物
    let wasm_path = find_wasm_output(package, target, release)?;
    println!(
        "{} wasm artifact: {}",
        "[VTX]".green(),
        wasm_path.display().to_string().yellow()
    );

    // 3) wasm -> component bytes
    println!("{}", "[VTX] Packaging VTX plugin...".cyan());
    let component_bytes = convert_to_component_bytes(&wasm_path)?;

    // 4) 只输出 .vtx
    let vtx_path = write_vtx_file(&wasm_path, &component_bytes)?;
    println!(
        "{} VTX package generated: {}",
        "[VTX]".green(),
        vtx_path.display().to_string().yellow()
    );

    Ok(())
}

/// 根据 package 名定位 wasm 输出文件。
///
/// Cargo 会把 crate 名中的 '-' 自动替换为 '_' 作为文件名：
/// vtx-plugin-auth-basic -> vtx_plugin_auth_basic.wasm
fn find_wasm_output(package: &str, target: &str, release: bool) -> Result<PathBuf> {
    let profile_dir = if release { "release" } else { "debug" };
    let dir = Path::new("target").join(target).join(profile_dir);

    let crate_name = package.replace('-', "_");

    let candidates = [
        format!("{crate_name}.wasm"),
        format!("lib{crate_name}.wasm"),
        format!("{package}.wasm"),
        format!("lib{package}.wasm"),
    ];

    for name in candidates {
        let p = dir.join(&name);
        if p.exists() {
            return Ok(p);
        }
    }

    // 兜底：扫描目录下所有 .wasm，选一个包含 crate_name 的
    let rd = std::fs::read_dir(&dir)
        .with_context(|| format!("failed to read dir: {}", dir.display()))?;

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
        "wasm output not found in: {} (tried common names + scan)",
        dir.display()
    );
}

/// wasm module bytes -> component bytes
///
/// 流程：
/// - 剥离 wit-bindgen 的 exports-removed 元数据段（避免 merge duplicate import）
/// - 自动选择 WASI preview1 adapter：
///   - 有 `_start` 导出 -> command adapter
///   - 没有 `_start` 导出 -> reactor adapter
fn convert_to_component_bytes(input_wasm_path: &Path) -> Result<Vec<u8>> {
    let module_bytes = std::fs::read(input_wasm_path).with_context(|| {
        format!(
            "failed to read wasm from: {}",
            input_wasm_path.display()
        )
    })?;

    let cleaned_module = strip_exports_removed_bindgen_section(&module_bytes)?;

    let (adapter_bytes, adapter_kind) = select_wasi_preview1_adapter(&cleaned_module)?;
    println!(
        "{} Using WASI preview1 {} adapter.",
        "[VTX]".dimmed(),
        adapter_kind.yellow()
    );

    let component_bytes = ComponentEncoder::default()
        .module(&cleaned_module)
        .context("Failed to encode module (ComponentEncoder::module)")?
        .adapter(WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, adapter_bytes)
        .context("Failed to add WASI preview1 adapter")?
        .validate(true)
        .encode()
        .context("Failed to finalize component encoding (ComponentEncoder::encode)")?;

    Ok(component_bytes)
}

fn select_wasi_preview1_adapter(module: &[u8]) -> Result<(&'static [u8], &'static str)> {
    let has_start = module_exports_start(module)?;
    if has_start {
        Ok((WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER, "command"))
    } else {
        Ok((WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER, "reactor"))
    }
}

/// 判断 wasm 是否导出 `_start`
fn module_exports_start(module: &[u8]) -> Result<bool> {
    let mut parser = WasmParser::new(0);
    let mut offset = 0usize;

    while offset < module.len() {
        let chunk = parser
            .parse(&module[offset..], true)
            .context("failed to parse wasm")?;

        let (consumed, payload) = match chunk {
            Chunk::NeedMoreData(_) => anyhow::bail!("unexpected NeedMoreData"),
            Chunk::Parsed { consumed, payload } => (consumed, payload),
        };

        offset += consumed;

        match payload {
            Payload::ExportSection(reader) => {
                for item in reader {
                    let export = item?;
                    if export.name == "_start" {
                        return Ok(true);
                    }
                }
            }
            Payload::End(_) => break,
            _ => {}
        }
    }

    Ok(false)
}

/// 只输出 `.vtx`：
/// `.vtx` 编码由 vtx-format 统一实现（VTX\x01 + component bytes）
fn write_vtx_file(input_wasm_path: &Path, component_bytes: &[u8]) -> Result<PathBuf> {
    let out_path = input_wasm_path.with_extension("vtx");

    let buf = vtx_format::encode_v1(component_bytes);

    std::fs::write(&out_path, buf)
        .with_context(|| format!("failed to write vtx: {}", out_path.display()))?;

    Ok(out_path)
}

/// 剥离 wit-bindgen 注入的 `with-all-of-its-exports-removed` 元数据段
fn strip_exports_removed_bindgen_section(module: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(module.len());

    let mut parser = WasmParser::new(0);
    let mut offset = 0usize;

    let mut removed = 0usize;
    let mut kept_component_type = 0usize;

    while offset < module.len() {
        let chunk = parser
            .parse(&module[offset..], true)
            .context("failed to parse wasm")?;

        let (consumed, payload) = match chunk {
            Chunk::NeedMoreData(_) => anyhow::bail!("unexpected NeedMoreData for in-memory parse"),
            Chunk::Parsed { consumed, payload } => (consumed, payload),
        };

        let raw = &module[offset..offset + consumed];

        let mut keep = true;
        if let Payload::CustomSection(cs) = &payload {
            let name = cs.name();

            if name.starts_with("component-type:wit-bindgen:")
                && name.contains("with-all-of-its-exports-removed")
            {
                removed += 1;
                keep = false;
            } else if name.starts_with("component-type:") {
                kept_component_type += 1;
            }
        }

        if keep {
            out.extend_from_slice(raw);
        }

        offset += consumed;

        if matches!(payload, Payload::End(_)) {
            break;
        }
    }

    if removed > 0 && kept_component_type == 0 {
        return Ok(module.to_vec());
    }

    if removed > 0 {
        println!(
            "{} Stripped {} wit-bindgen metadata section(s) (exports-removed).",
            "[VTX]".dimmed(),
            removed.to_string().yellow()
        );
    }

    Ok(out)
}
