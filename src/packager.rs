use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};
use wasmparser::{Chunk, Parser as WasmParser, Payload};
use wit_component::ComponentEncoder;

use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME,
    WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER,
    WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};

/// 核心流程：读取 Wasm -> 清理元数据 -> 注入 Adapter -> 编码组件
///
/// # 流程
/// - 读取 wasm 文件并清理元数据
/// - 根据模块的特点选择合适的 adapter
/// - 编码并返回组件字节
pub fn process_wasm(input_wasm_path: &Path) -> Result<Vec<u8>> {
    // 读取 Wasm 模块字节
    let module_bytes = std::fs::read(input_wasm_path).with_context(|| {
        format!("failed to read wasm from: {}", input_wasm_path.display())
    })?;

    // 1. 清理：去除 wit-bindgen 元数据
    let cleaned_module = strip_exports_removed_bindgen_section(&module_bytes)?;

    // 2. 选择适当的 Adapter（根据模块内容决定）
    let (adapter_bytes, adapter_kind) = select_wasi_preview1_adapter(&cleaned_module)?;
    println!(
        "{} Using WASI preview1 {} adapter",
        "[VTX]".dimmed(),
        adapter_kind.yellow()
    );

    // 3. 编码组件：将 Wasm 模块与 Adapter 编码成组件
    ComponentEncoder::default()
        .module(&cleaned_module)
        .context("Failed to encode module")?
        .adapter(WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, adapter_bytes)
        .context("Failed to add WASI preview1 adapter")?
        .validate(true) // 校验组件有效性
        .encode()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to finalize component encoding: {}\n\
                 Hint: Check wit-bindgen version compatibility or duplicate imports.",
                e
            )
        })
}

/// 写入 .vtx 文件
///
/// # 功能
/// - 将编码后的组件字节写入到 `.vtx` 文件
///
/// # 参数
/// - `input_wasm_path`: 输入的 wasm 文件路径
/// - `component_bytes`: 编码后的组件字节
pub fn write_vtx_file(input_wasm_path: &Path, component_bytes: &[u8]) -> Result<PathBuf> {
    // 生成输出路径，扩展名为 .vtx
    let out_path = input_wasm_path.with_extension("vtx");
    // 将组件字节编码为 VTX 格式并写入文件
    let buf = vtx_format::encode_v1(component_bytes);
    std::fs::write(&out_path, buf)
        .with_context(|| format!("failed to write vtx: {}", out_path.display()))?;
    Ok(out_path)
}

// --- 内部辅助函数 ---

/// 根据模块的内容选择 WASI Preview1 Adapter（Command 或 Reactor）
fn select_wasi_preview1_adapter(module: &[u8]) -> Result<(&'static [u8], &'static str)> {
    if module_exports_start(module)? {
        Ok((WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER, "command"))
    } else {
        Ok((WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER, "reactor"))
    }
}

/// 检查模块是否包含 `_start` 导出项
fn module_exports_start(module: &[u8]) -> Result<bool> {
    let mut parser = WasmParser::new(0);
    let mut offset = 0usize;

    while offset < module.len() {
        let chunk = parser.parse(&module[offset..], true)?;
        let (consumed, payload) = match chunk {
            Chunk::Parsed { consumed, payload } => (consumed, payload),
            _ => return Ok(false),
        };
        offset += consumed;

        if let Payload::ExportSection(reader) = payload {
            for item in reader {
                if item?.name == "_start" { return Ok(true); }
            }
        }
    }
    Ok(false)
}

/// 去除 wit-bindgen 元数据部分
fn strip_exports_removed_bindgen_section(module: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(module.len());
    let mut parser = WasmParser::new(0);
    let mut offset = 0usize;
    let mut removed = 0usize;
    let mut unhandled_warned = false;

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
            // 检查并移除 wit-bindgen 元数据部分
            if name.starts_with("component-type:wit-bindgen:") {
                if name.contains("with-all-of-its-exports-removed") {
                    removed += 1;
                    keep = false;
                } else if !unhandled_warned {
                    println!(
                        "{} WARN: Found unhandled wit-bindgen section: '{}' ",
                        "[VTX]".yellow(), name
                    );
                    unhandled_warned = true;
                }
            }
        }

        // 保留其他部分
        if keep { out.extend_from_slice(raw); }
        offset += consumed;
    }

    // 输出已移除的 wit-bindgen 元数据数量
    if removed > 0 {
        println!(
            "{} Stripped {} wit-bindgen metadata section(s).",
            "[VTX]".dimmed(), removed.to_string().yellow()
        );
    }
    Ok(out)
}
