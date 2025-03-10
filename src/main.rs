mod cli;
mod db;
mod fs;
mod utils;

use anyhow::Result;
use cli::{Commands, TagCommands};

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = cli::parse_cli();

    // Match command and execute appropriate function
    match cli.command {
        Commands::Completion { shell } => {
            cli::completion::completion(shell)?;
        }

        Commands::Push { path, tags } => {
            cli::push::push(&path, tags)?;
        }

        Commands::Pop { numbers, tags } => {
            cli::pop::pop(numbers, tags)?;
        }

        Commands::List { tags } => {
            cli::list::list(tags)?;
        }

        Commands::Tag(tag_cmd) => match tag_cmd {
            TagCommands::Add { number, tags } => {
                cli::tag::add_tags(number, tags)?;
            }

            TagCommands::Remove { number, tags } => {
                cli::tag::remove_tags(number, tags)?;
            }

            TagCommands::List | TagCommands::Ls => {
                cli::tag::list_tags()?;
            }
        },

        Commands::Remove { numbers, tags } => {
            cli::remove::remove(numbers, tags)?;
        }

        Commands::Restore { number, tags } => {
            cli::restore::restore(number, tags)?;
        }

        Commands::Peek { number, tags } => {
            cli::peek::peek(number, tags)?;
        }
    }

    Ok(())
}
