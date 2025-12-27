use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};
use wasmparser::{Chunk, Parser as WasmParser, Payload};
use wit_component::ComponentEncoder;

use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER,
    WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};

/// 核心打包流程：Wasm -> VTX Component
///
/// # 流程说明
/// 1. 读取原始 Wasm 二进制流。
/// 2. 清理非必要的元数据（如 wit-bindgen 残留），减小体积并避免冲突。
/// 3. 智能选择 Adapter（Command vs Reactor 模式）。
/// 4. 编码为 WebAssembly Component Model。
///
/// # 参数
/// - `input_wasm_path`: 原始 Wasm 文件路径。
///
/// # 返回值
/// - 成功：返回编码后的组件字节流。
/// - 失败：返回包含上下文的错误信息。
pub fn process_wasm(input_wasm_path: &Path) -> Result<Vec<u8>> {
    let module_bytes = std::fs::read(input_wasm_path).with_context(|| {
        format!(
            "Failed to read raw wasm from: {}",
            input_wasm_path.display()
        )
    })?;

    // 步骤 1: 元数据清理
    let cleaned_module = strip_exports_removed_bindgen_section(&module_bytes)?;

    // 步骤 2: Adapter 选择
    let (adapter_bytes, adapter_kind) = select_wasi_preview1_adapter(&cleaned_module)?;
    // 仅在 verbose 模式下建议开启此日志，此处保留关键信息
    println!("{} Adapting module as: {}", "[INFO]".dimmed(), adapter_kind);

    // 步骤 3: 组件编码
    // ComponentEncoder 负责将 Module + Adapter 链接为 Component
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

    Ok(component_bytes)
}

/// 写入 VTX 格式文件
///
/// # 行为
/// - 在输入文件同目录下生成 `.vtx` 文件。
/// - 使用 vtx-format 库进行封装（增加头部/版本信息）。
pub fn write_vtx_file(input_path: &Path, component_bytes: &[u8]) -> Result<PathBuf> {
    let out_path = input_path.with_extension("vtx");

    // 调用 vtx_format 库进行最终封装
    // 假设 vtx_format::encode_v1 负责添加 Magic Number 和 Version Header
    let buf = vtx_format::encode_v1(component_bytes);

    std::fs::write(&out_path, buf)
        .with_context(|| format!("Failed to write vtx artifact: {}", out_path.display()))?;

    Ok(out_path)
}

// --- 内部辅助逻辑 ---

/// 根据模块导出表特征选择 WASI Adapter
///
/// 规则：
/// - 包含 `_start` 导出 -> Command Adapter (类似可执行程序)
/// - 否则 -> Reactor Adapter (类似库/服务)
fn select_wasi_preview1_adapter(module: &[u8]) -> Result<(&'static [u8], &'static str)> {
    if module_exports_start(module)? {
        Ok((WASI_SNAPSHOT_PREVIEW1_COMMAND_ADAPTER, "command"))
    } else {
        Ok((WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER, "reactor"))
    }
}

/// 探测 Wasm 模块是否导出 `_start` 符号
///
/// 复杂度：线性扫描 Export Section，O(N)
fn module_exports_start(module: &[u8]) -> Result<bool> {
    let mut parser = WasmParser::new(0);
    let mut offset = 0usize;

    while offset < module.len() {
        let chunk = parser.parse(&module[offset..], true)?;
        let (consumed, payload) = match chunk {
            Chunk::Parsed { consumed, payload } => (consumed, payload),
            _ => return Ok(false), // 解析结束或错误
        };
        offset += consumed;

        if let Payload::ExportSection(reader) = payload {
            for item in reader {
                if item?.name == "_start" {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// 清理 wit-bindgen 生成的特定 Custom Section
///
/// 目的：解决由不同版本绑定工具生成的元数据冲突问题，确保 Adapter 注入成功。
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
            // 识别并移除导致冲突的 bindgen section
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
