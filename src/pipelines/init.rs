use anyhow::Result;
use colored::*;
use std::path::Path;

use crate::templates;

pub fn execute_init_pipeline(name: &str, language: &str) -> Result<()> {
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

    std::fs::write(
        project_dir.join("Cargo.toml"),
        templates::rust_cargo_toml(name),
    )?;
    std::fs::write(src_dir.join("lib.rs"), templates::rust_lib_rs())?;
    std::fs::write(project_dir.join("vtx.toml"), templates::rust_vtx_toml(name))?;

    Ok(())
}

fn init_ts(project_dir: &Path, name: &str) -> Result<()> {
    let src_dir = project_dir.join("src");
    let dist_dir = project_dir.join("dist");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::create_dir_all(&dist_dir)?;

    std::fs::write(
        project_dir.join("package.json"),
        templates::ts_package_json(name),
    )?;
    std::fs::write(src_dir.join("index.ts"), templates::ts_index_ts())?;
    std::fs::write(project_dir.join("vtx.toml"), templates::ts_vtx_toml(name))?;

    Ok(())
}

fn init_python(project_dir: &Path, name: &str) -> Result<()> {
    let src_dir = project_dir.join("src");
    let module_dir = src_dir.join(name.replace('-', "_"));
    let dist_dir = project_dir.join("dist");
    std::fs::create_dir_all(&module_dir)?;
    std::fs::create_dir_all(&dist_dir)?;

    std::fs::write(
        project_dir.join("pyproject.toml"),
        templates::pyproject_toml(name),
    )?;
    std::fs::write(module_dir.join("__init__.py"), templates::python_init_py())?;
    std::fs::write(
        project_dir.join("vtx.toml"),
        templates::python_vtx_toml(name),
    )?;

    Ok(())
}
