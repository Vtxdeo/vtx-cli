mod builder;
mod checker;
mod cli;
mod config;
mod packager;
mod pipelines;
mod templates;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use pipelines::{
    execute_build_pipeline, execute_check_pipeline, execute_init_pipeline, execute_package_pipeline,
};

/// VTX CLI Banner
const BANNER: &str = r#"
__      __  _______  __   __
\ \    / / |__   __| \ \ / /
 \ \  / /     | |     \ V /
  \ \/ /      | |      > <
   \  /       | |     / . \
    \/        |_|    /_/ \_\
"#;

/// CLI Entry Point
fn main() -> Result<()> {
    // Print the ASCII art banner first
    println!("{}", BANNER.green().bold());

    let cli = Cli::parse();

    // Catch top-level errors to format them nicely and avoid showing Rust stack traces
    if let Err(e) = run(cli) {
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}

/// Execute the main business logic
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Build {
            package,
            target,
            release,
            force,
            debug,
        } => execute_build_pipeline(package, &target, release, force, debug),
        Commands::Check { debug } => execute_check_pipeline(debug),
        Commands::Package {
            input,
            force,
            debug,
        } => execute_package_pipeline(&input, debug, force),
        Commands::Init { name, language } => execute_init_pipeline(&name, &language),
    }
}
