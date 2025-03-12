#!/bin/bash
set -e

# Check if a version argument was provided
if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

VERSION=$1

# Check if version format is valid
if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be in format X.Y.Z"
  exit 1
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
  echo "Error: Working directory is not clean. Please commit or stash your changes."
  exit 1
fi

echo "Preparing release v$VERSION..."

# Update version in Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Commit version change
git add Cargo.toml
git commit -m "chore: bump version to $VERSION"

# Create and push git tag
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin main
git push origin "v$VERSION"

echo "Release v$VERSION initiated!"
echo "GitHub Actions workflow will now:"
echo "1. Build binaries for different platforms"
echo "2. Create a GitHub release"
echo "3. Update the Homebrew formula at archsyscall/homebrew-fstk"
echo ""
echo "Check the progress at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[\/:]\(.*\)\.git/\1/')/actions"