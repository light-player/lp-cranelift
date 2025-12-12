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

    /// Specify input files or directories to test
    #[arg(required = true)]
    files: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Test(t) => {
            lp_glsl_filetests::run(t.verbose, &t.files)?;
        }
    }

    Ok(())
}
