use clap::{Parser, Subcommand};

/// LP-GLSL filetest utility.
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run GLSL filetests
    Test(TestOptions),
}

/// Run GLSL filetests
#[derive(Parser)]
struct TestOptions {
    /// Be more verbose
    #[arg(short, long)]
    verbose: bool,

    /// Specify input files or directories to test (default: all tests)
    files: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Test(t) => {
            // If no files specified, run all tests using glob pattern
            let files = if t.files.is_empty() {
                vec!["**/*.glsl".to_string()]
            } else {
                t.files
            };
            lp_glsl_filetests::run(t.verbose, &files)?;
        }
    }

    Ok(())
}
