# dot

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/username/dot)

A Git proxy for managing hidden directories with version control, written in Rust.

[中文文档](docs/README_zh.md) | [Examples](docs/examples.md) | [Troubleshooting](docs/troubleshooting.md)

## Overview

`dot` is a powerful CLI tool that allows you to manage hidden directories (like `.kiro`, `.config`, etc.) with version control while keeping them invisible on public GitHub pages. It works by maintaining separate Git repositories for each hidden directory and synchronizing operations across all repositories atomically.

### 🚀 Key Features

- **Multi-Repository Management**: Manage multiple hidden directories, each with its own Git repository
- **Atomic Operations**: All operations are atomic by default - either all repositories succeed or all are rolled back
- **GitHub Integration**: Automatically creates and manages hidden repositories in specified GitHub organizations
- **Global Index**: Maintains a global index of all projects and their associated hidden repositories
- **Transparent Cloning**: When cloning a project, automatically discovers and clones all associated hidden repositories
- **Smart Key Generation**: Generates unique repository keys based on project URLs and directory paths
- **Flexible Configuration**: JSON-based configuration with organization authorization

## 📦 Installation

### Quick Install (Recommended)

```bash
git clone https://github.com/username/dot.git
cd dot
make install
```

### Manual Installation

#### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Git](https://git-scm.com/)
- GitHub personal access token with repository permissions

#### From Source

```bash
git clone https://github.com/username/dot.git
cd dot
cargo install --path .
```

#### Using Homebrew (macOS)

```bash
brew tap username/dot
brew install dot
```

#### Development Setup

```bash
git clone https://github.com/username/dot.git
cd dot
make setup  # Install dependencies and run tests
```

## ⚙️ Setup

### Quick Setup (Recommended)

Run the interactive setup wizard:

```bash
dot setup
```

The wizard will guide you through:
1. Checking Git configuration
2. Getting your GitHub username
3. Selecting a GitHub organization
4. Creating the configuration file
5. Setting up the index repository

### Manual Setup

#### 1. Set GitHub Token (Optional)

If you need API access beyond your Git credentials:

```bash
export GITHUB_TOKEN="your_github_token_here"

# Make it permanent
echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.bashrc
# or for zsh
echo 'export GITHUB_TOKEN="your_github_token_here"' >> ~/.zshrc
```

#### 2. Configure Organizations

On first use, `dot` will create `~/.dot/dot.conf`. Edit this file to add authorized GitHub organizations:

```bash
# Create example configuration
make create-config

# Edit the configuration
nano ~/.dot/dot.conf
```

Configuration format:
```json
{
  "authorized_organizations": ["your-org", "another-org"],
  "default_organization": "your-org"
}
```

### 3. Verify Installation

```bash
dot --version
dot --help
```

## 🎯 Quick Start

```bash
# 1. Initialize a project with hidden directories
cd your-project
dot init .kiro .config

# 2. Add and commit files
echo "secret config" > .kiro/settings.json
dot add .
dot commit -m "Add hidden configuration"

# 3. Push to all repositories
dot push

# 4. Clone project elsewhere (gets everything)
cd /tmp
dot clone git@github.com:user/your-project.git
```

## 📖 Usage

### Initialize a Project

Initialize dot in your project with one or more hidden directories:

```bash
# Single hidden directory
dot init .kiro

# Multiple hidden directories
dot init .kiro .config .secrets

# With global flags
dot init .kiro --no-atomic  # Disable atomic operations
```

**What happens:**
- Checks if git is initialized (initializes if not)
- Verifies git remote origin is set
- Creates separate Git repositories for each hidden directory
- Registers the project in the global index
- Publishes hidden repositories to your configured GitHub organization

### Check Status

View the status of all repositories:

```bash
dot status

# Skip hidden repositories
dot status --skip-hidden
```

### Add Files

Add files to all relevant repositories:

```bash
# Add specific files
dot add file1.txt .kiro/config.json

# Add all changes
dot add .

# Skip hidden repositories
dot add . --skip-hidden
```

### Commit Changes

Commit changes to all repositories with the same message:

```bash
dot commit -m "Update configuration and add new features"

# Non-atomic mode (continue even if some fail)
dot commit -m "Update" --no-atomic
```

### Push Changes

Push all repositories to their remotes:

```bash
dot push

# Skip hidden repositories
dot push --skip-hidden

# Non-atomic mode
dot push --no-atomic
```

### Clone Projects

Clone a project and automatically get all its hidden repositories:

```bash
# Clone to default directory name
dot clone git@github.com:user/project.git

# Clone to specific directory
dot clone git@github.com:user/project.git my-project
```

### Global Flags

All commands support these flags:

| Flag | Description |
|------|-------------|
| `--skip-hidden` | Skip operations on hidden repositories |
| `--no-atomic` | Disable atomic behavior (continue even if some operations fail) |
| `--help` | Show help information |

## 🔧 How It Works

### Repository Keys

Each hidden directory gets a unique Repository Key:

```
Format: {base_key}/{directory_path}

Example:
- Main repo: git@github.com:user/project.git
- Hidden dir: .kiro
- Repository Key: github.com/user/project/.kiro
```

### Global Index

`dot` maintains a global `.index` repository in your GitHub organization that tracks:
- All registered projects
- Associated hidden repositories
- Metadata (creation time, git user, paths, etc.)

### Atomic Operations

All multi-repository operations are atomic by default:

1. **Execute Phase**: Operations performed on hidden repositories first, then main repository
2. **Rollback Phase**: If any operation fails, all completed operations are rolled back
3. **Success**: All operations complete successfully

### Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Main Repo     │    │  Hidden Repo 1   │    │  Hidden Repo 2  │
│  (Public)       │    │   (.kiro)        │    │   (.config)     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │  Global Index    │
                    │   (.index)       │
                    └──────────────────┘
```

## 📋 Configuration

### Configuration File

**Location**: `~/.dot/dot.conf`

**Format**:
```json
{
  "authorized_organizations": [
    "my-personal-org",
    "my-company-org"
  ],
  "default_organization": "my-personal-org"
}
```

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GITHUB_TOKEN` | GitHub personal access token | Yes |
| `GH_TOKEN` | Alternative to GITHUB_TOKEN | Yes (if GITHUB_TOKEN not set) |

### Makefile Commands

| Command | Description |
|---------|-------------|
| `make install` | Build and install globally |
| `make build` | Build debug version |
| `make release` | Build release version |
| `make test` | Run tests |
| `make clean` | Clean build artifacts |
| `make setup` | Initial development setup |
| `make check-install` | Verify installation |

## 📚 Examples

### Complete Workflow

```bash
# 1. Setup new project
mkdir my-project && cd my-project
git init
git remote add origin git@github.com:user/my-project.git

# 2. Initialize with hidden directories
dot init .kiro .config

# 3. Create content
echo "# My Project" > README.md
echo '{"theme": "dark"}' > .kiro/settings.json
echo 'debug=true' > .config/app.conf

# 4. Commit everything
dot add .
dot commit -m "Initial project setup"
dot push

# 5. Clone elsewhere (gets everything)
cd /tmp
dot clone git@github.com:user/my-project.git
cd my-project
ls -la  # Shows README.md, .kiro/, .config/
```

### Working with Existing Projects

```bash
# Check if project has dot initialized
dot status

# Initialize existing project
dot init .kiro

# Clone existing project with hidden repos
dot clone git@github.com:user/existing-project.git
```

### Advanced Usage

```bash
# Non-atomic operations (continue on failure)
dot add . --no-atomic
dot commit -m "Partial update" --no-atomic
dot push --no-atomic

# Skip hidden repositories
dot status --skip-hidden
dot push --skip-hidden

# Check configuration
make show-config
```

## 🐛 Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| "Organization not authorized" | Add organization to `~/.dot/dot.conf` |
| "GitHub API error" | Check `GITHUB_TOKEN` permissions |
| "git is not installed" | Install Git and add to PATH |
| "Invalid git remote origin URL" | Set remote: `git remote add origin <url>` |

### Debug Commands

```bash
# Check installation
make check-install

# Show configuration
make show-config

# Verify GitHub token
echo $GITHUB_TOKEN

# Test basic functionality
dot --version
dot --help
```

### Getting Help

1. Check the [documentation](docs/)
2. Look at [examples](docs/examples.md)
3. Read the [troubleshooting guide](docs/troubleshooting.md)
4. Open an [issue](https://github.com/username/dot/issues)

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development

```bash
# Setup development environment
git clone https://github.com/username/dot.git
cd dot
make setup

# Run tests
make test

# Format and lint
make fmt
make clippy

# Build
make build
```

### Project Structure

```
dot/
├── src/                 # Source code
│   ├── main.rs         # CLI entry point
│   ├── config.rs       # Configuration management
│   ├── index.rs        # Global index management
│   ├── repository.rs   # Repository operations
│   ├── atomic.rs       # Atomic operations
│   └── ...
├── docs/               # Documentation
├── Formula/            # Homebrew formula
├── Makefile           # Build automation
└── README.md          # This file
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [clap](https://clap.rs/) for CLI parsing
- GitHub integration via [octocrab](https://github.com/XAMPPRocky/octocrab)
- Git operations with [git2](https://github.com/rust-lang/git2-rs)

---

**Made with ❤️ in Rust**
