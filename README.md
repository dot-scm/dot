# dot

A git command proxy CLI tool written in Rust.

## Overview

`dot` is a transparent proxy for git commands. Any git command can be executed through `dot`, making it easy to extend or customize git workflows in the future.

## Installation

### Using Homebrew (macOS)

```bash
brew tap username/dot
brew install dot
```

### From Source

```bash
git clone https://github.com/username/dot.git
cd dot
cargo install --path .
```

## Usage

Use `dot` exactly like you would use `git`:

```bash
# Check status
dot status

# Add files
dot add .

# Commit changes
dot commit -m "Your commit message"

# Push to remote
dot push

# Pull from remote
dot pull

# View log
dot log --oneline

# Any git command works
dot <git-command> [arguments]
```

### Built-in Commands

```bash
# Show dot version
dot --version

# Show help
dot --help
```

## License

MIT License - see [LICENSE](LICENSE) for details.
