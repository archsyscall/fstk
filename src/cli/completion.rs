use anyhow::Result;
use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use std::io;

use crate::cli::Cli;

/// Generate shell completion scripts
pub fn generate_completion<G: Generator>(gen: G, cmd: &mut Command, name: &str) -> Result<()> {
    generate(gen, cmd, name, &mut io::stdout());
    Ok(())
}

/// Generate shell completion script for the given shell
pub fn completion(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    generate_completion(shell, &mut cmd, &bin_name)?;

    // Print instructions for how to install the completion script
    println!("\n# Shell completion script generated for {}", bin_name);

    match shell {
        Shell::Bash => {
            println!("# To use, add this to your ~/.bashrc or ~/.bash_profile:");
            println!("# source <(fstk completion bash)");
            println!("# Or save it to a file in the bash completions directory:");
            println!("# fstk completion bash > ~/.local/share/bash-completion/completions/fstk");
        }
        Shell::Zsh => {
            println!("# To use, add this to your ~/.zshrc:");
            println!("# source <(fstk completion zsh)");
            println!("# Or save it to a file in the zsh completions directory:");
            println!("# mkdir -p ~/.zsh/completions");
            println!("# fstk completion zsh > ~/.zsh/completions/_fstk");
            println!("# Then add to your ~/.zshrc:");
            println!("# fpath=(~/.zsh/completions $fpath)");
            println!("# autoload -U compinit && compinit");
        }
        Shell::Fish => {
            println!("# To use, save it to the fish completions directory:");
            println!("# fstk completion fish > ~/.config/fish/completions/fstk.fish");
        }
        Shell::PowerShell => {
            println!("# To use, save it to a file and source it from your PowerShell profile:");
            println!("# fstk completion powershell > fstk-completion.ps1");
            println!("# . ./fstk-completion.ps1");
        }
        Shell::Elvish => {
            println!("# To use, save it to your elvish config directory:");
            println!("# mkdir -p ~/.elvish/lib");
            println!("# fstk completion elvish > ~/.elvish/lib/fstk-completions.elv");
            println!("# Then add to your ~/.elvish/rc.elv:");
            println!("# use fstk-completions");
        }
        _ => {}
    }

    Ok(())
}
