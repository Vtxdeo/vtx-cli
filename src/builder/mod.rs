use crate::config::BuildConfig;
use anyhow::Result;
use std::path::PathBuf;

pub mod go;
pub mod lua;
pub mod php;
pub mod python;
pub mod rust;
pub mod ts;

/// Build pipeline interface.
///
/// This trait defines the standard lifecycle for turning source code into
/// an intermediate Wasm artifact. Implementations should be stateless or
/// explicitly document file system side effects.
pub trait Builder {
    /// Stage 1: environment pre-check.
    ///
    /// Verify the host environment meets build requirements.
    ///
    /// # Behavior
    /// - Use lightweight commands like `--version` to check toolchain presence.
    /// - If checks fail, return an error with concrete installation guidance.
    fn check_env(&self) -> Result<()>;

    /// Stage 2: build execution.
    ///
    /// Invoke the underlying toolchain for compilation.
    ///
    /// # Parameters
    /// - `package`: Package name used for targets or build scripts.
    /// - `target`: Target architecture identifier (e.g. wasm32-wasi).
    /// - `release`: Build mode; true for optimized release builds.
    ///
    /// # Side effects
    /// - Produces disk IO and intermediate artifacts.
    /// - May consume significant CPU/memory.
    /// - May write toolchain logs to stdout/stderr.
    fn build(&self, package: &str, target: &str, release: bool) -> Result<()>;

    /// Stage 3: artifact resolution.
    ///
    /// Locate the final Wasm output after the build finishes.
    ///
    /// # Returns
    /// - Success: absolute or execution-relative path.
    /// - Failure: error if file is missing or ambiguous.
    fn find_output(&self, package: &str, target: &str, release: bool) -> Result<PathBuf>;
}

pub fn create_builder(
    language: &str,
    build_config: Option<BuildConfig>,
) -> Result<Box<dyn Builder>> {
    match language.to_lowercase().as_str() {
        "rust" | "rs" => Ok(Box::new(rust::RustBuilder)),
        "go" | "tinygo" => Ok(Box::new(go::GoBuilder)),
        "ts" | "typescript" | "js" | "node" => Ok(Box::new(ts::TsBuilder::new(build_config))),
        "py" | "python" => Ok(Box::new(python::PythonBuilder::new(build_config))),
        "php" => Ok(Box::new(php::PhpBuilder::new(build_config))),
        "lua" => Ok(Box::new(lua::LuaBuilder::new(build_config))),
        unsupported => anyhow::bail!("Unsupported language identifier: {unsupported}"),
    }
}
