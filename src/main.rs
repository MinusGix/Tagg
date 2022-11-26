pub mod commands;
pub mod config;
pub mod state;
pub mod storage;
pub mod tagg;
pub mod util;

use clap::{Parser, Subcommand};
use config::Config;
use state::State;
use tagg::Tagg;

#[derive(Debug, Parser)]
#[command(name = "tagg")]
#[command(about = "A tagging program", long_about = None)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Get information about the currently registration-area
    Status {},
    /// Add a file to the registration-area
    #[command(arg_required_else_help = true)]
    Add {
        /// Some number of files which the comment and tags are applied to
        files: Vec<String>,
        /// Note information about the file for you to reference later
        #[arg(long, short)]
        comment: Option<String>,
        /// A list of space-separated tags
        #[arg(long, short, num_args = 1..)]
        tags: Vec<String>,
    },
    /// Drop files from the registration-area
    #[command(arg_required_else_help = true)]
    Drop {
        files: Vec<String>,
    },
    // /// Add a file immediately into the storage, without going into registration
    // #[command(arg_required_else_help = true)]
    // AddQ {
    //     /// Some number of files which the comment and tags are applied to
    //     files: Vec<String>,
    //     /// Note information about the file for you to reference later
    //     #[arg(long, short)]
    //     comment: Option<String>,
    //     /// A list of space-separated tags
    //     #[arg(long, short, num_args = 1..)]
    //     tags: Vec<String>,
    // },
    /// Move files from the registration-area to the storage area
    Commit {
        /// Don't actually remove any files from the registration-area, or rename any, or delete any  
        /// Should typically be used with verbose
        #[arg(long)]
        dry: bool,
        /// Don't remove the original file
        #[arg(long)]
        soft: bool,
    },
    #[command(arg_required_else_help = true)]
    AddTags {
        files: Vec<String>,
        #[arg(long, short, num_args = 1..)]
        tags: Vec<String>,
    },
    /// Set a comment on files.  
    /// Default title is 'comment'
    #[command(arg_required_else_help = true)]
    SetComment {
        files: Vec<String>,
        #[arg(long, short)]
        message: String,
        #[arg(long, short)]
        title: Option<String>,
    },
    /// Set the title of a single file
    #[command(arg_required_else_help = true)]
    SetTitle {
        file: String,
        message: String,
    },
    /// Search based on tags
    #[command(arg_required_else_help = true)]
    Find {
        tags: Vec<String>,
    },
    ListAll {},
    /// Open a file in the program assigned to it via xdg-open (on Linux)
    #[command(arg_required_else_help = true)]
    Open {
        /// The files to open, note that it opens them individually
        files: Vec<String>,
        /// The program to use in opening it
        #[arg(long, short)]
        using: Option<String>,
    },
}

fn main() -> eyre::Result<()> {
    let args = Cli::parse();

    let config_path = Config::config_path();
    if args.verbose {
        eprintln!(
            "Loading config from {:?}",
            config_path
                .canonicalize()
                .unwrap_or_else(|_| config_path.clone())
        );
    }
    let config = Config::load_from(&config_path).expect("Failed to load config file");

    let state_path = config.state_path(&config_path)?;
    if args.verbose {
        eprintln!(
            "Loading state from {:?}",
            state_path
                .canonicalize()
                .unwrap_or_else(|_| state_path.clone())
        );
    }
    let state = State::load_from(&state_path).expect("Failed to load state file");

    // TODO: Check that storage folder exists

    let mut tagg = Tagg {
        config_path,
        state_path,
        config,
        state,

        verbose: args.verbose,
    };

    commands::dispatch(&mut tagg, args.command)?;

    Ok(())
}
