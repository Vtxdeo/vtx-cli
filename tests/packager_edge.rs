use std::io::Write;
use tempfile::NamedTempFile;
use vtx_cli::packager::process_wasm;
use wit_component::ComponentEncoder;

const CORE_MODULE_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

fn make_component_bytes() -> anyhow::Result<Vec<u8>> {
    let encoder = ComponentEncoder::default().module(&CORE_MODULE_HEADER)?;
    let component = encoder.validate(false).encode()?;
    Ok(component)
}

fn write_temp(bytes: &[u8]) -> anyhow::Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(bytes)?;
    Ok(file)
}

#[test]
fn process_wasm_rejects_truncated_magic() -> anyhow::Result<()> {
    let file = write_temp(&[0x00, 0x61, 0x73, 0x6d])?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(err
        .to_string()
        .contains("Failed to parse wasm header for component detection"));
    Ok(())
}

#[test]
fn process_wasm_rejects_truncated_module_section() -> anyhow::Result<()> {
    let bytes = [
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, 0x01, // type section id + size, but missing payload
    ];
    let file = write_temp(&bytes)?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(!err.to_string().is_empty());
    Ok(())
}

#[test]
fn process_wasm_rejects_component_header_truncated_payload() -> anyhow::Result<()> {
    let bytes = [
        0x00, 0x61, 0x73, 0x6d, // magic
        0x0d, 0x00, 0x00, 0x00, // component version
        0x01, 0x01, // pretend section id + size, but missing payload
    ];
    let file = write_temp(&bytes)?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(!err.to_string().is_empty());
    Ok(())
}

#[test]
fn process_wasm_rejects_large_section_length() -> anyhow::Result<()> {
    let bytes = [
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, 0xff, 0xff, 0xff, 0xff, 0x0f, // type section, length = 0x1fffffff
    ];
    let file = write_temp(&bytes)?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(!err.to_string().is_empty());
    Ok(())
}

#[test]
fn process_wasm_rejects_magic_plus_noise() -> anyhow::Result<()> {
    let bytes = [
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // garbage payload
    ];
    let file = write_temp(&bytes)?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(!err.to_string().is_empty());
    Ok(())
}

#[test]
fn process_wasm_rejects_empty_input() -> anyhow::Result<()> {
    let file = write_temp(&[])?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(err
        .to_string()
        .contains("Failed to parse wasm header for component detection"));
    Ok(())
}

#[test]
fn process_wasm_rejects_garbage_input() -> anyhow::Result<()> {
    let file = write_temp(&[0xde, 0xad, 0xbe, 0xef])?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(err
        .to_string()
        .contains("Failed to parse wasm header for component detection"));
    Ok(())
}

#[test]
fn process_wasm_skips_encoding_for_component() -> anyhow::Result<()> {
    let component = make_component_bytes()?;
    let file = write_temp(&component)?;
    let output = process_wasm(file.path(), false, true)?;
    assert_eq!(output, component);
    Ok(())
}

#[test]
fn process_wasm_rejects_missing_contract_without_force() -> anyhow::Result<()> {
    let component = make_component_bytes()?;
    let file = write_temp(&component)?;
    let err = process_wasm(file.path(), false, false).unwrap_err();
    assert!(err.to_string().contains("Contract Violation"));
    Ok(())
}

#[test]
fn process_wasm_encodes_core_module_when_forced() -> anyhow::Result<()> {
    let file = write_temp(&CORE_MODULE_HEADER)?;
    let output = process_wasm(file.path(), false, true)?;
    assert!(output.len() > CORE_MODULE_HEADER.len());
    Ok(())
}
