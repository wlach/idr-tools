use std::env;

use clap::{Parser, Subcommand};
use idr_tools::create_idr;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new IDR
    New {
        title: String,
        /// Omit comment instructions from the generated IDR
        #[arg(long)]
        no_comments: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
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
