use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use crate::templates;

#[derive(Clone)]
struct InitContext {
    name: String,
    language: String,
}

pub fn execute_init_pipeline(
    name: Option<&str>,
    language: Option<&str>,
    interactive: bool,
) -> Result<()> {
    let mut ctx = InitContext {
        name: name.unwrap_or_default().trim().to_string(),
        language: language.unwrap_or_default().trim().to_string(),
    };

    if interactive || ctx.name.is_empty() || ctx.language.is_empty() {
        println!("{} Interactive init", "[VTX]".green().bold());
        ctx = prompt_init(ctx)?;
    }

    let language = normalize_language(&ctx.language);
    let name = ctx.name.trim();
    if name.is_empty() {
        anyhow::bail!("Project name cannot be empty.");
    }

    probe_environment(&language);

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Target directory already exists: {}", project_dir.display());
    }

    std::fs::create_dir_all(project_dir)?;

    match language.as_str() {
        "rust" => init_rust(project_dir, name),
        "ts" => init_ts(project_dir, name),
        "python" => init_python(project_dir, name),
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

fn prompt_init(mut ctx: InitContext) -> Result<InitContext> {
    let default_name = if ctx.name.is_empty() {
        "vtx-demo"
    } else {
        ctx.name.as_str()
    };

    let name = prompt_text("Project name", default_name)?;
    ctx.name = name;

    let default_language = if ctx.language.is_empty() {
        "rust"
    } else {
        ctx.language.as_str()
    };
    let language = prompt_language(default_language)?;
    ctx.language = language;

    Ok(ctx)
}

fn prompt_text(label: &str, default: &str) -> Result<String> {
    print!("{label} ({default}): ");
    io::stdout().flush()?;
    let input = read_line()?;
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

fn prompt_language(default: &str) -> Result<String> {
    println!("Select language:");
    println!("  1) Rust");
    println!("  2) TypeScript");
    println!("  3) Python");
    print!("Enter choice (default: {default}): ");
    io::stdout().flush()?;
    let input = read_line()?;

    let choice = input.trim();
    let language = match choice {
        "" => default.to_string(),
        "1" => "rust".to_string(),
        "2" => "ts".to_string(),
        "3" => "python".to_string(),
        other => other.to_string(),
    };

    Ok(normalize_language(&language))
}

fn read_line() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn normalize_language(language: &str) -> String {
    match language.to_lowercase().as_str() {
        "rust" | "rs" => "rust".to_string(),
        "ts" | "typescript" | "js" | "node" => "ts".to_string(),
        "py" | "python" => "python".to_string(),
        other => other.to_string(),
    }
}

fn probe_environment(language: &str) {
    match language {
        "rust" => probe_rust_environment(),
        _ => {}
    }
}

fn probe_rust_environment() {
    let cargo_ok = command_ok("cargo", &["--version"]);
    if cargo_ok {
        println!("{} Cargo detected.", "[OK]".green().bold());
    } else {
        println!(
            "{} Cargo not found. Install Rust from https://rustup.rs",
            "[WARN]".yellow()
        );
        return;
    }

    let rustup_ok = command_ok("rustup", &["--version"]);
    if !rustup_ok {
        println!(
            "{} rustup not found. Target checks skipped.",
            "[WARN]".yellow()
        );
        return;
    }

    match rustup_has_target("wasm32-wasip1") {
        Some(true) => println!("{} wasm32-wasip1 target installed.", "[OK]".green().bold()),
        Some(false) => println!(
            "{} wasm32-wasip1 target missing. Run: rustup target add wasm32-wasip1",
            "[WARN]".yellow()
        ),
        None => println!(
            "{} Unable to inspect installed targets.",
            "[WARN]".yellow()
        ),
    }
}

fn command_ok(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn rustup_has_target(target: &str) -> Option<bool> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().any(|line| line.trim() == target))
}
