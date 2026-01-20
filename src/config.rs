use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Project configuration structure.
/// Maps to vtx.toml in the project root.
#[derive(Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub vtx_version: Option<u32>,
    pub project: ProjectInfo,
    pub build: Option<BuildConfig>,
}

/// Project author information.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Base project metadata.
#[derive(Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    /// Plugin package name used for identification and artifact naming.
    pub name: String,

    /// Plugin version declared by the author.
    pub version: Option<String>,

    /// Project language identifier (e.g. rust, go, ts, python, php, lua).
    /// This determines which build strategy the CLI uses.
    pub language: String,

    /// Plugin author written into .vtx metadata for registry and security scanning.
    pub author: Option<String>,

    /// Author list (PEP 621 style).
    pub authors: Option<Vec<ProjectAuthor>>,

    /// Project description.
    pub description: Option<String>,

    /// License identifier.
    pub license: Option<String>,

    /// Homepage URL.
    pub homepage: Option<String>,

    /// Repository URL.
    pub repository: Option<String>,

    /// Keywords.
    pub keywords: Option<Vec<String>>,
}

/// Build configuration.
#[derive(Deserialize, Debug, Clone)]
pub struct BuildConfig {
    /// Custom build command to override default build logic.
    pub cmd: Option<String>,

    /// Custom output directory for build artifacts (.wasm).
    pub output_dir: Option<String>,

    /// Exact artifact file name.
    pub artifact: Option<String>,
}

/// Load and parse vtx.toml from the current directory.
///
/// # Boundaries
/// - Must be called from the project root or it returns an error.
/// - File size is expected to be in KB range; uses synchronous IO.
pub fn load() -> Result<ProjectConfig> {
    let config_path = Path::new("vtx.toml");

    if !config_path.exists() {
        anyhow::bail!("Configuration file 'vtx.toml' not found in current directory.");
    }

    let content = fs::read_to_string(config_path).context("Failed to read vtx.toml file")?;

    let config: ProjectConfig =
        toml::from_str(&content).context("Failed to parse vtx.toml content")?;

    if let Some(version) = config.vtx_version {
        if version != 1 {
            anyhow::bail!("Unsupported vtx.toml version: {version}");
        }
    }

    Ok(config)
}
