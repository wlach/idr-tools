use std::path::PathBuf;
use std::{env, fs};

use clap::{Parser, Subcommand};
use jiff::Timestamp;
use minijinja::{Environment, context};

mod git;
mod utils;

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

    let mut env = Environment::new();
    env.add_template("idr", include_str!("./default_template.md.jinja"))
        .unwrap();
    let tmpl = env.get_template("idr").unwrap();

    match &cli.command {
        Commands::New { title, no_comments } => {
            // Check environment variable for no_comments if flag not set
            let should_strip_comments = *no_comments || env::var("IDR_NO_COMMENTS").is_ok();

            let now = Timestamp::now();

            let now_yyyymmdd = now.strftime("%Y-%m-%d").to_string();
            let owner = git::get_identity().unwrap_or_else(|| "Unknown <unknown>".to_string());

            let mut rendered_output = tmpl
                .render(context!(
                    date => now_yyyymmdd,
                    title => title,
                    owner => owner,
                ))
                .unwrap();

            if should_strip_comments {
                rendered_output = utils::strip_html_comments(&rendered_output);
            }

            let current_dir: PathBuf = env::current_dir().unwrap();
            let path =
                utils::get_idr_path(current_dir.as_path(), utils::get_idr_filename(title, now));

            fs::write(path, rendered_output).expect("Failed to write idr");
        }
    }
}
