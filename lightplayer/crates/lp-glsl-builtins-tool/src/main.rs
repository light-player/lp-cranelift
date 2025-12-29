//! CLI tool for generating CLIF and registry code from Rust builtin implementations.
//!
//! This tool compiles `lp-glsl-builtins-src` with the Cranelift backend,
//! extracts `__lp_*` functions from the generated CLIF, validates and transforms them,
//! and generates both textual CLIF files and Rust registry code.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod generator;

#[derive(Parser)]
#[command(name = "lp-glsl-builtins-tool")]
#[command(about = "Generate CLIF and registry code from Rust builtin implementations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate textual CLIF files and registry code
    GenerateClif {
        /// Source directory containing Rust builtin implementations
        #[arg(long, default_value = "../lp-glsl-builtins-src")]
        source_dir: String,
        
        /// Output directory for generated CLIF files
        #[arg(long, default_value = "src/generated/clif")]
        output_dir: String,
        
        /// Output file for registry code
        #[arg(long, default_value = "src/generated/registry.rs")]
        registry_file: String,
    },
    
    /// Generate binary CLIF files
    GenerateBinaries {
        /// Directory containing textual CLIF files
        #[arg(long, default_value = "src/generated/clif")]
        clif_dir: String,
        
        /// Output directory for binary CLIF files
        #[arg(long, default_value = "src/generated/clif")]
        output_dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::GenerateClif { source_dir, output_dir, registry_file } => {
            generator::generate_clif(&source_dir, &output_dir, &registry_file)?;
        }
        Commands::GenerateBinaries { clif_dir, output_dir } => {
            generator::generate_binaries(&clif_dir, &output_dir)?;
        }
    }
    
    Ok(())
}

