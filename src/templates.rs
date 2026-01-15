pub fn rust_cargo_toml(name: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nvtx-sdk = \"0.1.8\"\nserde = {{ version = \"1.0\", features = [\"derive\"] }}\nserde_json = \"1.0\"\nanyhow = \"1.0\"\n"
    )
}

pub fn rust_lib_rs() -> String {
    "use vtx_sdk::{export_plugin, VtxPlugin};\n\n#[derive(Default)]\nstruct Plugin;\n\nimpl VtxPlugin for Plugin {}\n\nexport_plugin!(Plugin);\n"
        .to_string()
}

pub fn rust_vtx_toml(name: &str) -> String {
    let crate_name = name.replace('-', "_");
    format!(
        "[project]\nname = \"{name}\"\nlanguage = \"rust\"\n\n[build]\ncmd = \"cargo build --target wasm32-wasip1 --release\"\noutput_dir = \"target/wasm32-wasip1/release\"\nartifact = \"{crate_name}.wasm\"\n\n[sdk]\nversion = \"0.1.8\"\n"
    )
}

pub fn ts_package_json(name: &str) -> String {
    format!(
        "{{\n  \"name\": \"{name}\",\n  \"version\": \"0.1.0\",\n  \"scripts\": {{\n    \"build\": \"echo TODO: build wasm\"\n  }}\n}}\n"
    )
}

pub fn ts_index_ts() -> String {
    "export {};\n".to_string()
}

pub fn ts_vtx_toml(name: &str) -> String {
    format!(
        "[project]\nname = \"{name}\"\nlanguage = \"ts\"\n\n[build]\ncmd = \"npm run build\"\noutput_dir = \"dist\"\nartifact = \"{name}.wasm\"\n\n[sdk]\nversion = \"0.2.0\"\n"
    )
}

pub fn pyproject_toml(name: &str) -> String {
    format!(
        "[build-system]\nrequires = [\"setuptools\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[project]\nname = \"{name}\"\nversion = \"0.1.0\"\n"
    )
}

pub fn python_init_py() -> String {
    "# plugin entry\n".to_string()
}

pub fn python_vtx_toml(name: &str) -> String {
    let module_name = name.replace('-', "_");
    format!(
        "[project]\nname = \"{name}\"\nlanguage = \"python\"\n\n[build]\ncmd = \"componentize-py -d . -o dist/{name}.wasm {module_name}\"\noutput_dir = \"dist\"\nartifact = \"{name}.wasm\"\n\n[sdk]\nversion = \"0.2.0\"\n"
    )
}
