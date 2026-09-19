use std::path::PathBuf;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

pub const VERSION_STRING: &str = "v2026.09.19";

#[derive(Parser, Debug)]
#[command(
    name = "jev-ops",
    about = "Extensible AI-powered diagnostics for DevOps & SRE — analyze logs and infrastructure signals using pluggable packs and Jev",
    version = VERSION_STRING,
    author
)]
pub struct Cli {
    /// Increase logging verbosity (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze diagnostic input using a loaded pack
    Analyze {
        /// Name of the diagnostic pack to use
        pack: String,

        /// Path to input file (defaults to reading from standard input)
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Emit machine-consumable JSON output
        #[arg(long)]
        json: bool,

        /// Additional custom pack directory to search
        #[arg(long)]
        pack_dir: Option<PathBuf>,

        /// Flag decisions below confidence threshold (0.0 to 1.0)
        #[arg(long)]
        min_confidence: Option<f64>,

        /// Inference provider to use ('mock' or 'typesafe')
        #[arg(long)]
        provider: Option<String>,

        /// TypeSafe API key (overrides TYPESAFE_API_KEY environment variable)
        #[arg(long)]
        api_key: Option<String>,
    },

    /// Discover, inspect, validate, and scaffold diagnostic packs
    Packs {
        #[command(subcommand)]
        command: PacksCommands,
    },

    /// Generate shell auto-completions
    Completion {
        /// Target shell for completion script
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Display version information
    Version,
}

#[derive(Subcommand, Debug)]
pub enum PacksCommands {
    /// List all discovered diagnostic packs
    List {
        /// Additional custom pack directory to search
        #[arg(long)]
        pack_dir: Option<PathBuf>,

        /// Output listing as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show detailed metadata and decision schemas of a pack
    Show {
        /// Name of the pack
        pack: String,

        /// Additional custom pack directory to search
        #[arg(long)]
        pack_dir: Option<PathBuf>,

        /// Output details as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate a pack manifest file or directory against safety and schema rules
    Validate {
        /// Path to pack manifest (pack.yaml) or directory
        path: PathBuf,
    },

    /// Scaffold a new starter diagnostic pack
    New {
        /// Name of the pack to scaffold
        name: String,

        /// Target directory to scaffold into (defaults to ./packs)
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}
