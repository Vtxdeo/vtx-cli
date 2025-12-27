use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};
use wasmparser::{Chunk, Parser as WasmParser, Payload};
use wit_component::ComponentEncoder;

use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};

/// 核心打包流程：Wasm -> VTX Component
///
/// 流程说明：
/// 1. 读取原始 Wasm 二进制流。
/// 2. 清理非必要的元数据。
/// 3. 强制注入 Reactor Adapter。
/// 4. 编码为 WebAssembly Component Model。
/// 5. 执行契约校验。
///
/// 参数：
/// - `input_wasm_path`: 原始 Wasm 文件路径。
/// - `debug`: 是否输出详细调试信息。
/// - `force`: 校验失败时是否强制继续。
pub fn process_wasm(input_wasm_path: &Path, debug: bool, force: bool) -> Result<Vec<u8>> {
    let module_bytes = std::fs::read(input_wasm_path).with_context(|| {
        format!(
            "Failed to read raw wasm from: {}",
            input_wasm_path.display()
        )
    })?;

    // 步骤 1: 元数据清理
    let cleaned_module = strip_exports_removed_bindgen_section(&module_bytes)?;

    // 步骤 2: Adapter 注入
    // VTX 插件必须运行在 Reactor 模式下，强制使用 Reactor Adapter
    let adapter_bytes = WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER;
    if debug {
        println!("{} Injecting WASI Reactor Adapter", "[DEBUG]".dimmed());
    }

    // 步骤 3: 组件编码
    let component_bytes = ComponentEncoder::default()
        .module(&cleaned_module)
        .context("Failed to encode module into component")?
        .adapter(WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, adapter_bytes)
        .context("Failed to inject WASI preview1 adapter")?
        .validate(true)
        .encode()
        .map_err(|e| {
            anyhow::anyhow!(
                "Component encoding error: {}\nEnsure wit-bindgen version matches adapter requirements.",
                e
            )
        })?;

    // 步骤 4: 契约校验
    // 检查生成的组件是否符合 VTX Kernel 的接口要求
    if let Err(e) = validate_contract(&component_bytes, debug) {
        if force {
            println!(
                "{} Contract validation failed but --force is enabled: {}",
                "[WARN]".yellow(),
                e
            );
        } else {
            return Err(e);
        }
    }

    Ok(component_bytes)
}

/// 写入 VTX 格式文件
pub fn write_vtx_file(input_path: &Path, component_bytes: &[u8]) -> Result<PathBuf> {
    let out_path = input_path.with_extension("vtx");
    let buf = vtx_format::encode_v1(component_bytes);

    std::fs::write(&out_path, buf)
        .with_context(|| format!("Failed to write vtx artifact: {}", out_path.display()))?;

    Ok(out_path)
}

// --- 内部辅助逻辑 ---

/// 验证生成的组件是否导出了内核要求的接口
///
/// 检查项：
/// 1. 是否导出 `handle` (HTTP 处理入口)
/// 2. 是否导出 `get-manifest` (元数据获取入口)
fn validate_contract(component_bytes: &[u8], debug: bool) -> Result<()> {
    let mut parser = WasmParser::new(0);
    let mut found_handle = false;
    let mut found_manifest = false;

    // 解析组件导出表
    for payload in parser.parse_all(component_bytes) {
        if let Ok(Payload::ComponentExportSection(reader)) = payload {
            for export in reader {
                let export = export?;
                // 修正：直接访问元组结构体的第一个字段获取名称
                let name = export.name.0;

                if debug {
                    println!("{} Found export: {}", "[DEBUG]".dimmed(), name);
                }

                // 检查 WIT 定义的关键入口
                // 这些名字对应 SDK `world plugin` 中的 export 定义
                // 注意：根据 wit-bindgen 版本不同，可能会带有接口前缀，这里做模糊匹配
                match name {
                    "handle" | "vtx:api/plugin/handle" | "vtx:api/plugin#handle" => {
                        found_handle = true
                    }
                    "get-manifest"
                    | "vtx:api/plugin/get-manifest"
                    | "vtx:api/plugin#get-manifest" => found_manifest = true,
                    _ => {}
                }
            }
        }
    }

    if !found_handle {
        anyhow::bail!("Contract Violation: Missing required export 'handle'.\nEnsure you have implemented the Plugin trait and used 'vtx_sdk::export!(...)' macro.");
    }
    if !found_manifest {
        anyhow::bail!("Contract Violation: Missing required export 'get-manifest'.");
    }

    if debug {
        println!("{} Contract validation passed.", "[INFO]".cyan());
    }

    Ok(())
}

/// 清理 wit-bindgen 生成的特定 Custom Section
fn strip_exports_removed_bindgen_section(module: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(module.len());
    let mut parser = WasmParser::new(0);
    let mut offset = 0usize;

    while offset < module.len() {
        let chunk = parser.parse(&module[offset..], true)?;
        let (consumed, payload) = match chunk {
            Chunk::Parsed { consumed, payload } => (consumed, payload),
            _ => break,
        };

        let raw = &module[offset..offset + consumed];
        let mut keep = true;

        if let Payload::CustomSection(cs) = &payload {
            let name = cs.name();
            if name.starts_with("component-type:wit-bindgen:")
                && name.contains("with-all-of-its-exports-removed")
            {
                keep = false;
            }
        }

        if keep {
            out.extend_from_slice(raw);
        }
        offset += consumed;
    }

    Ok(out)
}
