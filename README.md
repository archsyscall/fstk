# fstk

> A modern file stack manager - the perfect place for your files and directories

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

fstk (short for File Stack) is a command-line utility that manages files and directories using stack-like operations. Unlike traditional file movement commands, fstk maintains a history of file locations and allows you to restore files to their original places at any time. Beyond basic file operations, it offers features like tagging for organization, batch operations, and a complete history tracking. The tool maintains a database of all managed files and their original locations, making it useful for both temporary file storage and long-term file organization. Whether you're reorganizing your workspace, managing multiple versions of files, or moving files between locations, fstk provides a structured approach to file management with its stack-based operations.


## Features

- Push files and directories to a stack (keeping original path information)
- Pop items from the stack when you need them
- Organize with tags for easy retrieval
- Easy search and filter capabilities
- Batch operations with number ranges and comma-separated values
- Shell completion support for bash, zsh, fish, and more
- Follows Unix philosophy with silent success and focused error messages

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Shell Completion](#shell-completion)
- [Examples](#examples)
- [Storage Details](#storage-details)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Using Homebrew (macOS/Linux)

```bash
# Add the tap and install
brew tap archsyscall/fstk
brew install fstk
```

### From Source

```bash
# Clone the repository
git clone https://github.com/archsyscall/fstk.git
cd fstk

# Build and install
cargo build --release
cargo install --path .
```

### Setup Shell Completion (Optional)

```bash
# Generate and install completion script for your shell
# (See "Shell Completion" section below for more details)
fstk completion bash > ~/.local/share/bash-completion/completions/fstk
```

## Quick Start

```bash
# Push a file to the stack with tags
fstk push document.pdf -t work,important

# List all items in your stack
fstk list

# Pop the most recent item from stack 
fstk pop

# Pop a specific item by number
fstk pop 3

# Tag management (tags must be specified with -t flag)
fstk tag add 2 -t project,urgent
fstk tag list
```

## Commands

### Core Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `push <PATH>` | `p` | Push a file or directory to the stack |
| `pop [NUMBERS]` | `po` | Pop item(s) from the stack and restore to current directory |
| `list` | `ls` | List all items in the stack |
| `remove <NUMBERS>` | `rm` | Remove item(s) without restoring |
| `restore [NUMBER]` | `res` | Restore an item without removing it from the stack |
| `peek [NUMBER]` | `pk` | Preview item's metadata without restoring |
| `tag` | - | Tag management commands |
| `completion <SHELL>` | - | Generate shell completion scripts |

### Tag Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `tag add <NUMBER>` | - | Add tags to an item (requires `-t` flag) |
| `tag remove <NUMBER>` | `tag rm` | Remove tags from an item (requires `-t` flag) |
| `tag list` | `tag l`, `tag ls` | List all tags with usage count |

### Common Options

| Option | Description |
|--------|-------------|
| `-t, --tags <TAGS>` | Specify comma-separated tags |
| `-h, --help` | Display help information |
| `-V, --version` | Display version information |

## Shell Completion

`fstk` supports shell completion for all major shells. This helps you tab-complete commands, options, and even arguments where possible.

### Bash

```bash
# Add to your current session
source <(fstk completion bash)

# Or install permanently
fstk completion bash > ~/.local/share/bash-completion/completions/fstk
```

### Zsh

```bash
# Create completions directory if it doesn't exist
mkdir -p ~/.zsh/completions

# Generate and save the completion script
fstk completion zsh > ~/.zsh/completions/_fstk

# Add to your ~/.zshrc
fpath=(~/.zsh/completions $fpath)
autoload -U compinit && compinit
```

### Fish

```bash
fstk completion fish > ~/.config/fish/completions/fstk.fish
```

## Examples

### Basic Usage

```bash
# Push multiple files with tags (-t flag with comma-separated tags)
fstk push important_doc.pdf -t work,priority
fstk push ~/projects/demo/ -t project,demo

# List everything
fstk list

# List items with specific tags (-t flag with comma-separated tags)
fstk list -t project

# Pop the most recent item (restores to current directory)
fstk pop

# Pop specific items by number (restores to current directory)
fstk pop 2
```

### Batch Operations

```bash
# Pop multiple items at once
fstk pop 1,3,5      # Pop items 1, 3, and 5
fstk pop 1-5        # Pop items 1 through 5
fstk pop 1,3-5,7    # Pop items 1, 3 through 5, and 7

# Remove multiple items
fstk rm 1,3-5,7
```

### Tag Management

```bash
# Add tags to an item (requires -t flag with comma-separated tags)
fstk tag add 2 -t urgent,followup

# Remove tags from an item (requires -t flag with comma-separated tags) 
fstk tag remove 2 -t followup  # or short form: fstk tag rm 2 -t followup

# List all tags with usage count
fstk tag list
```

## Storage Details

`fstk` keeps your data organized in predictable locations:

- **Data files:** `~/.fstk/.data/` - Where your stored files and directories are kept
- **Database:** `~/.fstk/fstk.db` (SQLite) - Tracks metadata, tags, and original locations

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.# fstk
