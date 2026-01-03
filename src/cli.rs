use clap::{Parser, Subcommand};

/// Main CLI struct
/// Parses command line arguments and handles command dispatch
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Subcommands available for the CLI
    #[command(subcommand)]
    pub command: Commands,
}

/// Supported subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Build and package the plugin (wasm -> component -> .vtx)
    Build {
        /// Workspace package name. If not specified, it will be read from vtx.toml.
        #[arg(short, long)]
        package: Option<String>,

        /// Build target architecture (default: "wasm32-wasip1")
        #[arg(long, default_value = "wasm32-wasip1")]
        target: String,

        /// Enable release mode (optimized build)
        #[arg(long, default_value_t = true)]
        release: bool,

        /// Force mode: Ignore SDK version mismatches or non-fatal contract errors
        #[arg(long, default_value_t = false)]
        force: bool,

        /// Debug mode: Retain debug symbols and output verbose logs
        #[arg(long, default_value_t = false)]
        debug: bool,
    },

    /// Check environment and configuration without building
    Check {
        /// Debug mode: Output verbose check logs
        #[arg(long, default_value_t = false)]
        debug: bool,
    },

    /// Package an existing Wasm artifact into .vtx format
    Package {
        /// Input Wasm file path
        #[arg(short, long)]
        input: String,

        /// Force mode: Ignore non-fatal contract errors
        #[arg(long, default_value_t = false)]
        force: bool,

        /// Debug mode: Output verbose packaging logs
        #[arg(long, default_value_t = false)]
        debug: bool,
    },

    /// Initialize a new plugin project scaffold
    Init {
        /// Project name (creates a new directory)
        #[arg(short, long)]
        name: String,

        /// Language (rust|ts|python)
        #[arg(short, long)]
        language: String,
    },
}
