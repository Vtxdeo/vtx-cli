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
    format!(
        "vtx_version = 1\n\n[project]\nname = \"{name}\"\nversion = \"0.1.0\"\nlanguage = \"rust\"\nauthors = [{{ name = \"Your Name\", email = \"you@example.com\" }}]\ndescription = \"Short plugin summary\"\nlicense = \"MIT\"\nhomepage = \"https://example.com\"\nrepository = \"https://example.com/repo\"\nkeywords = [\"vtx\", \"plugin\"]\n"
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
        "vtx_version = 1\n\n[project]\nname = \"{name}\"\nversion = \"0.1.0\"\nlanguage = \"ts\"\nauthors = [{{ name = \"Your Name\", email = \"you@example.com\" }}]\ndescription = \"Short plugin summary\"\nlicense = \"MIT\"\nhomepage = \"https://example.com\"\nrepository = \"https://example.com/repo\"\nkeywords = [\"vtx\", \"plugin\"]\n"
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
    format!(
        "vtx_version = 1\n\n[project]\nname = \"{name}\"\nversion = \"0.1.0\"\nlanguage = \"python\"\nauthors = [{{ name = \"Your Name\", email = \"you@example.com\" }}]\ndescription = \"Short plugin summary\"\nlicense = \"MIT\"\nhomepage = \"https://example.com\"\nrepository = \"https://example.com/repo\"\nkeywords = [\"vtx\", \"plugin\"]\n"
    )
}
