# Contributing to fstk

First of all, thank you for considering contributing to fstk! It's people like you that make this tool better.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report. Following these guidelines helps maintainers understand your report, reproduce the behavior, and find related reports.

* **Use the GitHub issue search** — check if the issue has already been reported.
* **Check if the issue has been fixed** — try to reproduce it using the latest `main` branch.
* **Use the bug report template** — when you create a new issue, you will be presented with a template. Please fill it out as completely as possible.

### Suggesting Features

This section guides you through submitting a feature suggestion, including completely new features and minor improvements to existing functionality.

* **Use the GitHub issue search** — check if the feature has already been suggested.
* **Use the feature request template** — when you create a new issue, select the feature request template and fill it out.

### Pull Requests

The process described here has several goals:

- Maintain code quality
- Fix problems that are important to users
- Enable a sustainable system for maintainers to review contributions

Please follow these steps to have your contribution considered by the maintainers:

1. Follow all instructions in the template
2. Follow the [styleguides](#styleguides)
3. After you submit your pull request, verify that all [status checks](https://help.github.com/articles/about-status-checks/) are passing

## Styleguides

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line

### Rust Styleguide

* Use `cargo fmt` to format your code
* Run `cargo clippy` and address all warnings
* Ensure your code passes all tests with `cargo test`
* Add tests for new functionality
* Write documentation for public API functions

### Documentation Styleguide

* Use Markdown for documentation
* Keep comments in code concise and relevant
* Update README.md if needed with new features

## Additional Notes

### Issue and Pull Request Labels

We use labels to help us organize and track issues and pull requests:

* `bug` - Indicates an issue representing a bug in fstk
* `documentation` - Indicates a issue or PR that improves documentation
* `enhancement` - Indicates a issue or PR that adds new functionality
* `help wanted` - Indicates that a maintainer wants help on an issue or PR
* `good first issue` - Good for newcomers

### Code of Conduct

In the interest of fostering an open and welcoming environment, we expect all participants to be respectful and considerate. Harassment, trolling, and other disruptive behaviors will not be tolerated.