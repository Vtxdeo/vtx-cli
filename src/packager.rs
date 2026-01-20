use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};
use wasmparser::{Chunk, Encoding, Parser as WasmParser, Payload};
use wit_component::ComponentEncoder;

use wasi_preview1_component_adapter_provider::{
    WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER,
};

/// Core packaging flow: Wasm -> VTX Component.
///
/// Flow:
/// 1. Read raw Wasm bytes.
/// 2. Strip non-essential metadata.
/// 3. Check user imports (warnings only).
/// 4. Inject Reactor Adapter.
/// 5. Encode into WebAssembly Component Model.
/// 6. Validate exports contract.
///
/// Parameters:
/// - `input_wasm_path`: Raw Wasm file path.
/// - `debug`: Whether to emit verbose logs.
/// - `force`: Whether to continue on contract validation failures.
pub fn process_wasm(input_wasm_path: &Path, debug: bool, force: bool) -> Result<Vec<u8>> {
    let module_bytes = std::fs::read(input_wasm_path).with_context(|| {
        format!(
            "Failed to read raw wasm from: {}",
            input_wasm_path.display()
        )
    })?;

    // Fast path: already a component, skip adapter injection and encoding.
    if is_component(&module_bytes)
        .with_context(|| "Failed to parse wasm header for component detection")?
    {
        println!(
            "{} Input is already a WebAssembly component; skipping adapter injection and encoding.",
            "[INFO]".cyan()
        );

        validate_contract_with_force(&module_bytes, debug, force)?;

        return Ok(module_bytes);
    }

    // Step 1: metadata cleanup.
    // The cleaned module represents the user's compiled core logic.
    let cleaned_module = strip_exports_removed_bindgen_section(&module_bytes)?;

    // Step 2: dependency safety scan (Import Check).
    // Even with force=false, this only warns to keep builds open.
    validate_user_imports(&cleaned_module, debug);

    // Step 3: adapter injection.
    // VTX plugins must run in reactor mode, so inject the reactor adapter.
    let adapter_bytes = WASI_SNAPSHOT_PREVIEW1_REACTOR_ADAPTER;
    if debug {
        println!("{} Injecting WASI Reactor Adapter", "[DEBUG]".dimmed());
    }

    // Step 4: component encoding.
    let component_bytes = ComponentEncoder::default()
        .module(&cleaned_module)
        .context("Failed to encode module into component")?
        .adapter(WASI_SNAPSHOT_PREVIEW1_ADAPTER_NAME, adapter_bytes)
        .context("Failed to inject WASI preview1 adapter")?
        .validate(true)
        .encode()
        .map_err(|e| {
            anyhow::anyhow!(
                "Component encoding error: {e}\nEnsure wit-bindgen version matches adapter requirements."
            )
        })?;

    // Step 5: contract validation (Export Check).
    // Ensure the generated component matches VTX Kernel interfaces.
    validate_contract_with_force(&component_bytes, debug, force)?;

    Ok(component_bytes)
}

/// Write a VTX format file.
pub fn write_vtx_file(
    input_path: &Path,
    component_bytes: &[u8],
    metadata_json: &[u8],
) -> Result<PathBuf> {
    let out_path = input_path.with_extension("vtx");
    let buf = vtx_format::encode_v2(component_bytes, metadata_json);

    std::fs::write(&out_path, buf)
        .with_context(|| format!("Failed to write vtx artifact: {}", out_path.display()))?;

    Ok(out_path)
}

// --- Internal helpers ---

/// Validate that user module imports are in the trusted allowlist.
///
/// Purpose:
/// Detect host function dependencies that the kernel may not support.
/// Use a trust-but-verify approach and warn on unknown imports.
fn validate_user_imports(module_bytes: &[u8], debug: bool) {
    let parser = WasmParser::new(0);

    // Trusted namespace prefixes for import modules.
    // Any import module starting with these is considered safe or adapter-handled.
    let trusted_namespaces = [
        "wasi_snapshot_preview1", // Standard WASI Preview 1.
        "wasi:",                  // Standard WASI (Component Model).
        "vtx:",                   // VTX SDK official interface.
        "vtx",                    // VTX SDK legacy compatibility.
        "__wbindgen_",            // Rust wasm-bindgen internal intrinsics.
    ];

    for payload in parser.parse_all(module_bytes).flatten() {
        if let Payload::ImportSection(reader) = payload {
            for import in reader.into_iter().flatten() {
                let module = import.module;
                let field = import.name;

                // Check if the module is trusted.
                let is_trusted = trusted_namespaces.iter().any(|ns| module.starts_with(ns));

                if !is_trusted {
                    println!(
                        "{} Unknown Import Detected: '{}::{}'\n  \
                            {} This interface is not part of the standard VTX Kernel or WASI spec.\n  \
                            If the kernel does not provide this host function, the plugin will crash at runtime.",
                        "[WARN]".yellow(),
                        module,
                        field,
                        "->".yellow()
                    );
                } else if debug {
                    println!(
                        "{} Trusted import: {}::{}",
                        "[DEBUG]".dimmed(),
                        module,
                        field
                    );
                }
            }
        }
    }
}

fn validate_contract_with_force(component_bytes: &[u8], debug: bool, force: bool) -> Result<()> {
    if let Err(e) = validate_contract(component_bytes, debug) {
        if force {
            println!(
                "{} Contract validation failed but --force is enabled: {}",
                "[WARN]".yellow(),
                e
            );
            return Ok(());
        }
        return Err(e);
    }

    Ok(())
}

/// Determine whether the input is already a WebAssembly Component.
fn is_component(bytes: &[u8]) -> Result<bool> {
    let parser = WasmParser::new(0);

    for payload in parser.parse_all(bytes) {
        let payload = payload?;
        if let Payload::Version { encoding, .. } = payload {
            return Ok(matches!(encoding, Encoding::Component));
        }
    }

    Ok(false)
}

/// Validate that the generated component exports required kernel interfaces.
///
/// Checks:
/// 1. Export `handle` (HTTP entrypoint).
/// 2. Export `get-manifest` (metadata entrypoint).
fn validate_contract(component_bytes: &[u8], debug: bool) -> Result<()> {
    let parser = WasmParser::new(0);
    let mut found_handle = false;
    let mut found_manifest = false;
    let mut found_capabilities = false;

    // Parse component exports.
    for payload in parser.parse_all(component_bytes).flatten() {
        if let Payload::ComponentExportSection(reader) = payload {
            for export in reader {
                let export = export?;
                // Access the first tuple field to get the name.
                let name = export.name.0;

                if debug {
                    println!("{} Found export: {}", "[DEBUG]".dimmed(), name);
                }

                // Check WIT-defined entrypoints.
                // These names map to exports in the SDK `world plugin` definition.
                match name {
                    "handle" | "vtx:api/plugin/handle" | "vtx:api/plugin#handle" => {
                        found_handle = true
                    }
                    "get-manifest"
                    | "vtx:api/plugin/get-manifest"
                    | "vtx:api/plugin#get-manifest" => found_manifest = true,
                    "get-capabilities"
                    | "vtx:api/plugin/get-capabilities"
                    | "vtx:api/plugin#get-capabilities" => found_capabilities = true,
                    _ => {}
                }
            }
        }
    }

    if !found_handle {
        anyhow::bail!("Contract Violation: Missing required export 'handle'.\nEnsure you have implemented the Plugin trait and used 'vtx_sdk::export_plugin!(...)' macro.");
    }
    if !found_manifest {
        anyhow::bail!("Contract Violation: Missing required export 'get-manifest'.");
    }
    if !found_capabilities {
        anyhow::bail!("Contract Violation: Missing required export 'get-capabilities'.");
    }

    if debug {
        println!("{} Contract validation passed.", "[INFO]".cyan());
    }

    Ok(())
}

/// Remove specific custom sections generated by wit-bindgen.
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
