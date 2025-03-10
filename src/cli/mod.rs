pub mod completion;
pub mod list;
pub mod peek;
pub mod pop;
pub mod push;
pub mod remove;
pub mod restore;
pub mod tag;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fstk")]
#[command(about = "File Stack - A CLI tool for managing files and directories in a stack format")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate shell completion scripts
    #[command(about = "Generate shell completion scripts")]
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Push a file or directory to the stack
    #[command(alias = "p")]
    Push {
        /// Path to the file or directory to push
        path: String,

        /// Tags to associate with the pushed item (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// Pop an item from the stack and restore it to the current directory
    #[command(alias = "po")]
    Pop {
        /// Pop specific item(s) by number (as shown in the list command)
        /// Supports individual numbers (1), comma-separated lists (1,3,5), and ranges (1-5)
        #[arg(index = 1)]
        numbers: Option<String>,

        /// Pop the most recent item with the specified tags (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// List all items in the stack
    #[command(alias = "ls")]
    List {
        /// Filter by tags (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// Tag management commands
    #[command(subcommand)]
    Tag(TagCommands),

    /// Remove an item from the stack without restoring it
    #[command(alias = "rm")]
    Remove {
        /// Number(s) of the item(s) to remove (as shown in the list command)
        /// Supports individual numbers (1), comma-separated lists (1,3,5), and ranges (1-5)
        #[arg(index = 1)]
        numbers: String,

        /// Remove the items matching these numbers with the specified tags (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// Restore an item from the stack without removing it
    #[command(alias = "res")]
    Restore {
        /// Number of the item to restore (as shown in the list command)
        #[arg(index = 1)]
        number: Option<usize>,

        /// Restore the most recent item with the specified tags (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },

    /// Preview an item's metadata without restoring it
    #[command(alias = "pk")]
    Peek {
        /// Number of the item to peek (as shown in the list command)
        #[arg(index = 1)]
        number: Option<usize>,

        /// Peek the most recent item with the specified tags (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Option<Vec<String>>,
    },
}

#[derive(Subcommand)]
pub enum TagCommands {
    /// Add tags to an item
    #[command(alias = "a")]
    Add {
        /// Number of the item to tag (as shown in the list command)
        #[arg(index = 1)]
        number: usize,

        /// Tags to add (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// Remove tags from an item
    #[command(alias = "rm")]
    Remove {
        /// Number of the item to remove tags from (as shown in the list command)
        #[arg(index = 1)]
        number: usize,

        /// Tags to remove (comma-separated)
        #[arg(long, short = 't', value_delimiter = ',')]
        tags: Vec<String>,
    },

    /// List all tags
    #[command(visible_alias = "l")]
    List,

    /// Alias for 'list' (automatically added by clap)
    Ls,
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}
