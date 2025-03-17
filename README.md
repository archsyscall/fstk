# fstk

> Stack-based file & directory manager: Modern "cut/paste" alternative to mv

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

fstk (File Stack) is a command-line utility that manages files and directories using a stack-based approach. **This tool, which essentially works like "cut" operations**, allows you to place files in a stack (cut) and retrieve them to any desired location (paste). Unlike the traditional file moving command (mv), fstk maintains a history of file locations. The core functionality of fstk is that **you can retrieve files or directories stored with push to any desired location using the pop command**. You can move to your desired directory and run the pop command to bring the file to that location. It also provides tag-based organization, batch operations, and complete history tracking. Additionally, the restore command allows you to restore files to their original location without removing them from the stack, making fstk a convenient alternative to the mv command for file management.

## Features

- Manage files and directories with enhanced "cut/paste" functionality
- A safer alternative to the mv command for file and directory management
- Provides a more intuitive workflow by separating file movement into push and pop operations
- Push files and directories to a stack (preserving original path information)
- Pop items from the stack when needed (can be restored to original or any desired location)
- Simplifies file movement workflow when used with directory jumping tools like zoxide (z)
- Easy organization and search with tags
- Batch operations with number ranges and comma-separated values

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Commands](#commands)
- [Shell Completion](#shell-completion)
- [Storage Details](#storage-details)
- [License](#license)

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap archsyscall/fstk
brew install fstk
```

### From Source

```bash
git clone https://github.com/archsyscall/fstk.git
cd fstk
cargo build --release
cargo install --path .
```

## Usage

### Efficient Workflow (Using with zoxide)

```bash
# The mv command requires specifying both source and destination at once
mv report.pdf ~/projects/presentation/assets/

# Using fstk with zoxide makes this task simpler
fstk push report.pdf  # Add the file to the stack
z assets            # Quickly jump to the target directory using zoxide
fstk pop            # Restore the file to the current location
```

Advantages of this approach:
- No need to enter both file path and destination path in a single command
- Quickly navigate to complex paths with z without having to remember or type them
- Ability to push multiple files and pop them in different locations as needed

### Basic Usage

```bash
# Add a file to the stack (with tag options)
fstk push document.pdf -t work,important

# View stack contents
fstk list
fstk list -t project  # Filter by tag

# Retrieve files (restore to current directory)
fstk pop      # Most recent item
fstk pop 3    # Specific item number

# Retrieve files to a specific location
cd ~/different-directory/
fstk pop      # Restore to current location (~/different-directory/)
fstk restore 2 # Restore to original location while maintaining the item in the stack
```

### Tag Management

```bash
# Add tags to an item
fstk tag add 2 -t urgent,followup

# Remove tags from an item
fstk tag rm 2 -t followup

# View all tags
fstk tag list
```

### Batch Operations

```bash
# Retrieve multiple items at once
fstk pop 1,3,5      # Items 1, 3, and 5
fstk pop 1-5        # Items 1 through 5
fstk pop 1,3-5,7    # Items 1, 3 through 5, and 7

# Remove multiple items (without restoring)
fstk rm 1,3-5,7
```

## Commands

| Command | Description |
|---------|-------------|
| `push <PATH>` | Add file/directory to the stack |
| `pop [NUMBER]` | Remove item from the stack and restore to current directory (when executed in a different directory, restores to that location) |
| `list` | List all items in the stack |
| `remove <NUMBER>`