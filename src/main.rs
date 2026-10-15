use std::env;
use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use idr_tools::create_idr;
use idr_tools::lint::lint_idr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the raw IDR template, including guidance comments
    Template,
    /// Create a new IDR
    New {
        title: String,
        /// Omit comment instructions from the generated IDR
        #[arg(long)]
        no_comments: bool,
    },
    /// Check explicitly named IDRs for unfinished draft material
    Lint {
        #[arg(required = true, num_args = 1..)]
        files: Vec<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Template => print!("{}", idr_tools::idr::IDR_TEMPLATE),
        Commands::Lint { files } => {
            let mut status = 0;
            for path in files {
                let result = fs::read_to_string(path)
                    .map_err(anyhow::Error::from)
                    .and_then(|text| lint_idr(&text));
                match result {
                    Ok(findings) => {
                        if !findings.is_empty() {
                            status = status.max(1);
                        }
                        for finding in findings {
                            println!(
                                "{}:{}: {} {}",
                                path.display(),
                                finding.line,
                                finding.code,
                                finding.message
                            );
                        }
                    }
                    Err(error) => {
                        eprintln!("{}: {error:#}", path.display());
                        status = 2;
                    }
                }
            }
            std::process::exit(status);
        }
        Commands::New { title, no_comments } => {
            // Check environment variable for no_comments if flag not set
            let should_strip_comments = *no_comments || env::var("IDR_NO_COMMENTS").is_ok();

            let current_dir = env::current_dir().expect("Failed to get current directory");

            match create_idr(current_dir, title, should_strip_comments) {
                Ok(path) => {
                    println!("Created IDR: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to create IDR: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
